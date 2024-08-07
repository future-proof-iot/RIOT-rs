/// Generates a call to the macro defined in arch-specific driver modules, which needs to be
/// provided with a "list" of peripherals and their association interrupt(s).
///
/// This "list" is automatically generated from machine-readable data provided by
/// [`stm32-data-generated`](https://github.com/embassy-rs/stm32-data-generated).
/// Its schema is given by
/// [`stm32-data-serde`](https://github.com/embassy-rs/stm32-data/tree/main/stm32-data-serde)
/// (currently not published on crates.io).
// FIXME: rename this macro
#[proc_macro]
pub fn define_stm32_drivers(item: TokenStream) -> TokenStream {
    use std::fs;

    use quote::{format_ident, quote};
    use stm32_data_serde::Chip;

    #[allow(clippy::wildcard_imports)]
    use define_stm32_drivers::*;

    let input = syn::parse_macro_input!(item as Input);

    let chip_embassy_name = std::env::var("EMBASSY_CHIP").unwrap();
    // FIXME: how to obtain/distribute these files?
    let json_file_path = format!("../stm32-data-generated/data/chips/{chip_embassy_name}.json");
    let json = fs::read_to_string(json_file_path).unwrap();
    let chip: Chip = serde_json::from_str(&json).unwrap();

    let cores = chip.cores.iter();
    let mut peripherals = cores.flat_map(|c| c.peripherals.iter()).collect::<Vec<_>>();
    // Sort to make deduplication possible and make the generated "list" sorted by peripheral name.
    peripherals.sort_unstable_by_key(|p| &p.name);
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
            let peripheral_interrupts = relevant_peripherals.map(SpiPeripheral::from_peripheral);
            peripheral_interrupts
                .map(|p| {
                    let interrupt = format_ident!("{}", p.interrupt);
                    let name = format_ident!("{}", p.name);
                    quote! { #interrupt => #name }
                })
                .collect()
        }
        PeripheralKind::I2c => {
            let peripheral_interrupts = relevant_peripherals.map(I2cPeripheral::from_peripheral);
            peripheral_interrupts
                .map(|p| {
                    let ev_interrupt = format_ident!("{}", p.ev_interrupt);
                    let er_interrupt = format_ident!("{}", p.er_interrupt);
                    let name = format_ident!("{}", p.name);
                    quote! { #ev_interrupt, #er_interrupt => #name }
                })
                .collect()
        }
    };

    let peripheral_definition_macro = match input.peripheral_kind {
        PeripheralKind::Spi => format_ident!("define_spi_drivers"),
        PeripheralKind::I2c => format_ident!("define_i2c_drivers"),
    };

    // Some peripherals may not be usable at the same time because they rely on the same
    // interrupt. We do not currently handle these in this macro, they will however be detected
    // later in the compilation as peripherals attempt to move out the same interrupt.
    // TODO: we could either generate all driver definitions, individually gated on laze
    // `context`s, or only generate the definition needed for the current context/chip
    let expanded = quote! {
        #peripheral_definition_macro!(
            #( #peripheral_definitions ),*
        );
    };

    TokenStream::from(expanded)
}

mod define_stm32_drivers {
    use stm32_data_serde::chip::core;
    use syn::parse::{Parse, ParseStream};

    #[derive(Debug)]
    pub struct Input {
        pub peripheral_kind: PeripheralKind,
    }

    impl Parse for Input {
        fn parse(input: ParseStream) -> Result<Self, syn::Error> {
            let peripheral_kind = syn::Ident::parse(input)?;
            let peripheral_kind = PeripheralKind::try_from_ident(&peripheral_kind).unwrap();
            Ok(Self { peripheral_kind })
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

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SpiPeripheral {
        pub name: String,
        pub interrupt: String,
    }

    impl SpiPeripheral {
        /// Extracts data from the `stm32-data-serde` schema.
        pub fn from_peripheral(peripheral: &core::Peripheral) -> Self {
            let mut interrupts = peripheral.interrupts.as_ref().unwrap().iter();

            Self {
                name: peripheral.name.clone(),
                interrupt: interrupts
                    .find(|int| int.signal == "GLOBAL")
                    .unwrap()
                    .interrupt
                    .clone(),
            }
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
}
