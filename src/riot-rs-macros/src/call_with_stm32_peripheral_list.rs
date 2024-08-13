/// Generates a call to the given macro and provides it with a "list" of peripherals and optionally
/// their associated interrupt(s).
///
/// We take the macro to call as parameter because nested macros are evaluated started from the
/// outer one, and it thus would not work for this macro to only return the generated "list".
///
/// The "list" is automatically generated from machine-readable data provided by
/// [`stm32-data-generated`](https://github.com/embassy-rs/stm32-data-generated).
/// Its schema is given by
/// [`stm32-data-serde`](https://github.com/embassy-rs/stm32-data/tree/main/stm32-data-serde)
/// (currently not published on crates.io).
// FIXME: rename this macro
#[proc_macro]
pub fn call_with_stm32_peripheral_list(item: TokenStream) -> TokenStream {
    use std::fs;

    #[allow(clippy::wildcard_imports)]
    use call_with_stm32_peripheral_list::*;

    let input = syn::parse_macro_input!(item as Input);

    let chip_embassy_name = std::env::var("EMBASSY_CHIP").unwrap();
    // FIXME: how to obtain/distribute these files?
    let json_file_path = format!("../stm32-data-generated/data/chips/{chip_embassy_name}.json");
    let json = fs::read_to_string(json_file_path).unwrap();

    generate_stm32_driver_definition(&json, &chip_embassy_name, &input).into()
}

mod call_with_stm32_peripheral_list {
    use proc_macro2::TokenStream;
    use quote::{format_ident, quote};
    use stm32_data_serde::{chip::core, Chip};
    use syn::parse::{Parse, ParseStream};

    pub fn generate_stm32_driver_definition(
        json: &str,
        chip_embassy_name: &str,
        input: &Input,
    ) -> TokenStream {
        let chip: Chip = serde_json::from_str(json).unwrap();
        assert_eq!(chip.name, chip_embassy_name);

        let cores = chip.cores.iter();
        let mut peripherals = cores.flat_map(|c| c.peripherals.iter()).collect::<Vec<_>>();
        // Sort to make deduplication possible and make the generated "list" sorted by peripheral name.
        peripherals.sort_unstable_by_key(|p| &p.name);
        // FIXME: maybe add an assert to check this (check that every core has the same number of
        // peripherals)?
        // This assumes that every core can access every peripheral.
        peripherals.dedup_by_key(|p| &p.name);

        // Keep only the peripherals of the relevant kind.
        let relevant_peripherals = peripherals.into_iter().filter(|p| {
            if let Some(registers) = &p.registers {
                registers.kind == input.peripheral_kind.to_embassy_kind()
            } else {
                false
            }
        });

        // TODO: factor out the branches even more if possible
        // Collecting in each branch is required for type-erasure.
        let peripheral_definitions: Vec<_> = match input.peripheral_kind {
            PeripheralKind::Spi => {
                let peripheral_interrupts =
                    relevant_peripherals.filter_map(SpiPeripheral::try_from_peripheral);
                peripheral_interrupts
                    .map(|p| {
                        let interrupt = format_ident!("{}", p.interrupt);
                        let name = format_ident!("{}", p.name);

                        match input.output_kind {
                            OutputKind::Peripherals => quote! { #name },
                            OutputKind::PeripheralsAndInterrupts => quote! { #interrupt => #name },
                        }
                    })
                    .collect()
            }
            PeripheralKind::I2c => {
                let peripheral_interrupts =
                    relevant_peripherals.map(I2cPeripheral::from_peripheral);
                peripheral_interrupts
                    .map(|p| {
                        let ev_interrupt = format_ident!("{}", p.ev_interrupt);
                        let er_interrupt = format_ident!("{}", p.er_interrupt);
                        let name = format_ident!("{}", p.name);

                        match input.output_kind {
                            OutputKind::Peripherals => quote! { #name },
                            OutputKind::PeripheralsAndInterrupts => {
                                quote! { #ev_interrupt + #er_interrupt => #name }
                            }
                        }
                    })
                    .collect()
            }
        };

        let macro_to_call = &input.macro_to_call;

        // Some peripherals may not be usable at the same time because they rely on the same
        // interrupt. We do not currently handle these in this macro, they will however be detected
        // later in the compilation as peripherals attempt to move out the same interrupt.
        // TODO: we could either generate all driver definitions, individually gated on laze
        // `context`s, or only generate the definition needed for the current context/chip
        quote! {
            #macro_to_call!( #( #peripheral_definitions ),* );
        }
    }

    #[derive(Debug)]
    pub struct Input {
        pub macro_to_call: syn::Ident,
        pub peripheral_kind: PeripheralKind,
        pub output_kind: OutputKind,
    }

    impl Parse for Input {
        fn parse(input: ParseStream) -> Result<Self, syn::Error> {
            let macro_to_call = syn::Ident::parse(input)?;
            let _: syn::Token![!] = input.parse()?;

            let _: syn::Token![,] = input.parse()?;
            let peripheral_kind = syn::Ident::parse(input)?;
            let peripheral_kind = PeripheralKind::try_from_ident(&peripheral_kind).unwrap();

            let _: syn::Token![,] = input.parse()?;
            let output_kind = syn::Ident::parse(input)?;
            let output_kind = OutputKind::try_from_ident(&output_kind).unwrap();

            Ok(Self {
                macro_to_call,
                peripheral_kind,
                output_kind,
            })
        }
    }

    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub enum PeripheralKind {
        I2c,
        Spi,
    }

    impl PeripheralKind {
        pub fn to_embassy_kind(self) -> &'static str {
            match self {
                Self::Spi => "spi",
                Self::I2c => "i2c",
            }
        }

        pub fn try_from_ident(ident: &syn::Ident) -> Option<Self> {
            match ident.to_string().as_ref() {
                "Spi" => Some(Self::Spi),
                "I2c" => Some(Self::I2c),
                _ => None,
            }
        }
    }

    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub enum OutputKind {
        Peripherals,
        PeripheralsAndInterrupts,
    }

    impl OutputKind {
        pub fn try_from_ident(ident: &syn::Ident) -> Option<Self> {
            match ident.to_string().as_ref() {
                "Peripherals" => Some(Self::Peripherals),
                "PeripheralsAndInterrupts" => Some(Self::PeripheralsAndInterrupts),
                _ => None,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SpiPeripheral {
        pub name: String,
        pub interrupt: String,
    }

    impl SpiPeripheral {
        /// Extracts data from the `stm32-data-serde` schema.
        pub fn try_from_peripheral(peripheral: &core::Peripheral) -> Option<Self> {
            let mut interrupts = peripheral.interrupts.as_ref().unwrap().iter();

            let interrupt = interrupts.next().unwrap();
            // Assert that all SPI peripheral have exactly one interrupt.
            assert!(interrupts.next().is_none());

            let interrupt = match interrupt.signal.as_ref() {
                "RADIO" => return None, // This is for the SUBGHZSPI peripheral.
                "GLOBAL" => interrupt.interrupt.clone(),
                _ => panic!(
                    "{} is not a recognized SPI interrupt signal",
                    interrupt.signal
                ),
            };

            Some(Self {
                name: peripheral.name.clone(),
                interrupt,
            })
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct I2cPeripheral {
        pub name: String,
        pub ev_interrupt: String,
        pub er_interrupt: String,
    }

    impl I2cPeripheral {
        /// Extracts data from the `stm32-data-serde` schema.
        pub fn from_peripheral(peripheral: &core::Peripheral) -> Self {
            let mut interrupts = peripheral.interrupts.as_ref().unwrap().iter();

            I2cPeripheral {
                name: peripheral.name.clone(),
                ev_interrupt: interrupts
                    .clone()
                    .find(|int| int.signal == "EV")
                    .unwrap()
                    .interrupt
                    .clone(),
                er_interrupt: interrupts
                    .find(|int| int.signal == "ER")
                    .unwrap()
                    .interrupt
                    .clone(),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use std::fs;

        use super::*;

        fn test_driver_definition_from_json_file(
            chip_embassy_name: &str,
            peripheral_kind: PeripheralKind,
            output_kind: OutputKind,
            macro_to_call: syn::Ident,
            expected: &str,
        ) {
            let json_file_path =
                format!("../../../stm32-data-generated/data/chips/{chip_embassy_name}.json");
            let json = fs::read_to_string(&json_file_path).unwrap();
            let input = Input {
                macro_to_call,
                peripheral_kind,
                output_kind,
            };

            let generated =
                generate_stm32_driver_definition(&json, chip_embassy_name, &input).to_string();
            assert_eq!(generated, expected);
        }

        #[test]
        fn test_i2c_driver_definitions() {
            let macro_to_call = format_ident!("test");

            test_driver_definition_from_json_file(
                "STM32H755ZI",
                PeripheralKind::I2c,
                OutputKind::PeripheralsAndInterrupts,
                macro_to_call.clone(),
                "test ! (I2C1_EV + I2C1_ER => I2C1 , I2C2_EV + I2C2_ER => I2C2 , I2C3_EV + I2C3_ER => I2C3 , I2C4_EV + I2C4_ER => I2C4) ;",
            );
            test_driver_definition_from_json_file(
                "STM32H755ZI",
                PeripheralKind::I2c,
                OutputKind::Peripherals,
                macro_to_call.clone(),
                "test ! (I2C1 , I2C2 , I2C3 , I2C4) ;",
            );

            // There is no I2C2 on this chip.
            test_driver_definition_from_json_file(
                "STM32WB55RG",
                PeripheralKind::I2c,
                OutputKind::PeripheralsAndInterrupts,
                macro_to_call.clone(),
                "test ! (I2C1_EV + I2C1_ER => I2C1 , I2C3_EV + I2C3_ER => I2C3) ;",
            );
            test_driver_definition_from_json_file(
                "STM32WB55RG",
                PeripheralKind::I2c,
                OutputKind::Peripherals,
                macro_to_call.clone(),
                "test ! (I2C1 , I2C3) ;",
            );
        }

        #[test]
        fn test_spi_driver_definitions() {
            let macro_to_call = format_ident!("test");

            test_driver_definition_from_json_file(
                "STM32H755ZI",
                PeripheralKind::Spi,
                OutputKind::PeripheralsAndInterrupts,
                macro_to_call.clone(),
                "test ! (SPI1 => SPI1 , SPI2 => SPI2 , SPI3 => SPI3 , SPI4 => SPI4 , SPI5 => SPI5 , SPI6 => SPI6) ;",
            );
            test_driver_definition_from_json_file(
                "STM32H755ZI",
                PeripheralKind::Spi,
                OutputKind::Peripherals,
                macro_to_call.clone(),
                "test ! (SPI1 , SPI2 , SPI3 , SPI4 , SPI5 , SPI6) ;",
            );

            test_driver_definition_from_json_file(
                "STM32WB55RG",
                PeripheralKind::Spi,
                OutputKind::PeripheralsAndInterrupts,
                macro_to_call.clone(),
                "test ! (SPI1 => SPI1 , SPI2 => SPI2) ;",
            );
            test_driver_definition_from_json_file(
                "STM32WB55RG",
                PeripheralKind::Spi,
                OutputKind::Peripherals,
                macro_to_call.clone(),
                "test ! (SPI1 , SPI2) ;",
            );
        }

        // Test that JSON files generate something without panicking.
        #[test]
        fn test_all_json_files() {
            let json_files = fs::read_dir("../../../stm32-data-generated/data/chips").unwrap();

            for json_file_path in json_files {
                let json_file_path = json_file_path.unwrap().path();

                let json = fs::read_to_string(&json_file_path).unwrap();

                let chip_embassy_name = json_file_path.file_stem().unwrap().to_str().unwrap();
                dbg!(&chip_embassy_name);

                let input = Input {
                    macro_to_call: format_ident!("test"),
                    peripheral_kind: PeripheralKind::I2c,
                    output_kind: OutputKind::PeripheralsAndInterrupts,
                };
                let _generated =
                    generate_stm32_driver_definition(&json, &chip_embassy_name, &input).to_string();

                let input = Input {
                    macro_to_call: format_ident!("test"),
                    peripheral_kind: PeripheralKind::Spi,
                    output_kind: OutputKind::PeripheralsAndInterrupts,
                };
                let _generated =
                    generate_stm32_driver_definition(&json, &chip_embassy_name, &input).to_string();

                // We do not test with `OutputKind::Peripherals` as it is only a subset of
                // `OutputKind::PeripheralsAndInterrupts`.
            }
        }
    }
}
