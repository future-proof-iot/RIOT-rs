use embassy_stm32::{
    bind_interrupts,
    i2c::{
        ErrorInterruptHandler, EventInterruptHandler, I2c as InnerI2c, RxDma, SclPin, SdaPin, TxDma,
    },
    mode::Async,
    peripherals,
    time::Hertz,
    Peripheral,
};
use embedded_hal_async::i2c::Operation;
use riot_rs_macros::call_with_stm32_peripheral_list;

use crate::i2c::impl_async_i2c_for_driver_enum;

#[non_exhaustive]
#[derive(Clone)]
pub struct Config {
    pub frequency: Frequency,
    pub sda_pullup: bool,
    pub scl_pullup: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frequency: Frequency::K100,
            sda_pullup: false,
            scl_pullup: false,
        }
    }
}

// FIXME: check how well this matches the STM32 capabilities
// TODO: allow more free-from values?
#[derive(Copy, Clone)]
#[repr(u32)]
pub enum Frequency {
    K100 = 100_000,
    K250 = 250_000,
    K400 = 400_000,
    M1 = 1_000_000,
}

impl From<Frequency> for Hertz {
    fn from(freq: Frequency) -> Self {
        match freq {
            Frequency::K100 => Hertz::khz(100),
            Frequency::K250 => Hertz::khz(250),
            Frequency::K400 => Hertz::khz(400),
            Frequency::M1 => Hertz::mhz(1),
        }
    }
}

macro_rules! define_i2c_drivers {
    ($( $ev_interrupt:ident + $er_interrupt:ident => $peripheral:ident ),* $(,)?) => {
        // paste allows to create new identifiers by concatenation using `[<foo bar>]`.
        paste::paste! {
            $(
                /// Peripheral-specific I2C driver.
                // NOTE(arch): this is not required on this architecture, as the inner I2C type is
                // not generic over the I2C peripheral, and is only done for consistency with
                // other architectures.
                pub struct [<I2c $peripheral>] {
                    twim: InnerI2c<'static, Async>
                }

                impl [<I2c $peripheral>] {
                    #[must_use]
                    pub fn new(
                        twim_peripheral: impl Peripheral<P = peripherals::$peripheral> + 'static,
                        sda_pin: impl Peripheral<P: SdaPin<peripherals::$peripheral>> + 'static,
                        scl_pin: impl Peripheral<P: SclPin<peripherals::$peripheral>> + 'static,
                        tx_dma: impl Peripheral<P: TxDma<peripherals::$peripheral>> + 'static,
                        rx_dma: impl Peripheral<P: RxDma<peripherals::$peripheral>> + 'static,
                        config: Config,
                    ) -> Self {
                        let mut i2c_config = embassy_stm32::i2c::Config::default();
                        i2c_config.sda_pullup = config.sda_pullup;
                        i2c_config.scl_pullup = config.scl_pullup;

                        bind_interrupts!(
                            struct Irqs {
                                $ev_interrupt => EventInterruptHandler<peripherals::$peripheral>;
                                $er_interrupt => ErrorInterruptHandler<peripherals::$peripheral>;
                            }
                        );

                        let frequency = config.frequency;
                        let i2c = InnerI2c::new(
                            twim_peripheral,
                            scl_pin,
                            sda_pin,
                            Irqs,
                            tx_dma,
                            rx_dma,
                            frequency.into(),
                            i2c_config,
                        );

                        Self { twim: i2c }
                    }
                }
            )*

            /// Peripheral-agnostic driver.
            pub enum I2c {
                $( $peripheral([<I2c $peripheral>]), )*
            }

            impl embedded_hal_async::i2c::ErrorType for I2c {
                type Error = embassy_stm32::i2c::Error;
            }

            impl_async_i2c_for_driver_enum!(I2c, $( $peripheral ),*);
        }
    }
}

// Define a driver per peripheral
call_with_stm32_peripheral_list!(define_i2c_drivers!, I2c, PeripheralsAndInterrupts);
