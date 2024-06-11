use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice as InnerI2cDevice;
use embassy_rp::{
    bind_interrupts,
    i2c::{InterruptHandler, SclPin, SdaPin},
    peripherals,
};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embedded_hal_async::i2c::Operation;

use crate::i2c::impl_async_i2c_for_driver_enum;

// TODO: factor this out (across archs)?
// TODO: do we need a CriticalSectionRawMutex here?
pub type I2cDevice = InnerI2cDevice<'static, CriticalSectionRawMutex, I2c>;

// We do not provide configuration for internal pull-ups as the RP2040 datasheet mentions in
// section 4.3.1.3 that the GPIO used should have pull-ups enabled.
#[derive(Clone)]
#[non_exhaustive]
pub struct Config {
    pub frequency: Frequency,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frequency: Frequency::K100,
        }
    }
}

// Possible values are copied from embassy-nrf
// TODO: check how well this matches the RP2040 capabilities
#[derive(Copy, Clone)]
#[repr(u32)]
pub enum Frequency {
    K100 = 100_000,
    K250 = 250_000,
    K400 = 400_000,
}

macro_rules! define_i2c_drivers {
    ($( $interrupt:ident => $peripheral:ident ),* $(,)?) => {
        // paste allows to create new identifiers by concatenation using `[<foo bar>]`.
        paste::paste! {
            $(
                pub struct [<I2c $peripheral>] {
                    twim: embassy_rp::i2c::I2c<'static, peripherals::$peripheral, embassy_rp::i2c::Async>,
                }

                impl [<I2c $peripheral>] {
                    #[must_use]
                    pub fn new(
                        i2c_peripheral: peripherals::$peripheral,
                        sda_pin: impl SdaPin<peripherals::$peripheral>,
                        scl_pin: impl SclPin<peripherals::$peripheral>,
                        config: Config,
                    ) -> Self {
                        let mut i2c_config = embassy_rp::i2c::Config::default();
                        i2c_config.frequency = config.frequency as u32;

                        bind_interrupts!(
                            struct Irqs {
                                $interrupt => InterruptHandler<peripherals::$peripheral>;
                            }
                        );

                        let i2c =
                            embassy_rp::i2c::I2c::new_async(i2c_peripheral, scl_pin, sda_pin, Irqs, i2c_config);

                        Self { twim: i2c }
                    }
                }
            )*

            pub enum I2c {
                $( $peripheral([<I2c $peripheral>]), )*
            }

            impl embedded_hal_async::i2c::ErrorType for I2c {
                type Error = embassy_rp::i2c::Error;
            }

            impl_async_i2c_for_driver_enum!(I2c, $( $peripheral ),*);
        }
    }
}

// Define a driver per peripheral
define_i2c_drivers!(
    I2C0_IRQ => I2C0,
    I2C1_IRQ => I2C1,
);
