use embassy_nrf::{
    bind_interrupts,
    gpio::Pin as GpioPin,
    peripherals,
    twim::{InterruptHandler, Twim},
};
use embedded_hal_async::i2c::Operation;

use crate::i2c::impl_async_i2c_for_driver_enum;

pub use embassy_nrf::twim::Frequency;

#[non_exhaustive]
#[derive(Clone)]
pub struct Config {
    pub frequency: Frequency,
    pub sda_pullup: bool,
    pub scl_pullup: bool,
    pub sda_high_drive: bool,
    pub scl_high_drive: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frequency: Frequency::K100,
            sda_pullup: false,
            scl_pullup: false,
            sda_high_drive: false,
            scl_high_drive: false,
        }
    }
}

macro_rules! define_i2c_drivers {
    ($( $interrupt:ident => $peripheral:ident ),* $(,)?) => {
        // paste allows to create new identifiers by concatenation using `[<foo bar>]`.
        paste::paste! {
            $(
                /// Peripheral-specific I2C driver.
                pub struct [<I2c $peripheral>] {
                    twim: Twim<'static, peripherals::$peripheral>,
                }

                impl [<I2c $peripheral>] {
                    #[must_use]
                    pub fn new(
                        twim_peripheral: peripherals::$peripheral,
                        sda_pin: impl GpioPin,
                        scl_pin: impl GpioPin,
                        config: Config,
                    ) -> Self {
                        let mut twim_config = embassy_nrf::twim::Config::default();
                        twim_config.frequency = config.frequency;
                        twim_config.sda_pullup = config.sda_pullup;
                        twim_config.scl_pullup = config.scl_pullup;
                        twim_config.sda_high_drive = config.sda_high_drive;
                        twim_config.scl_high_drive = config.scl_high_drive;

                        bind_interrupts!(
                            struct Irqs {
                                $interrupt => InterruptHandler<peripherals::$peripheral>;
                            }
                        );

                        let twim = Twim::new(twim_peripheral, Irqs, sda_pin, scl_pin, twim_config);

                        Self { twim }
                    }
                }
            )*

            /// Peripheral-agnostic driver.
            pub enum I2c {
                $( $peripheral([<I2c $peripheral>]), )*
            }

            impl embedded_hal_async::i2c::ErrorType for I2c {
                type Error = embassy_nrf::twim::Error;
            }

            impl_async_i2c_for_driver_enum!(I2c, $( $peripheral ),*);
        }
    }
}

// FIXME: support other nRF archs
// Define a driver per peripheral
#[cfg(context = "nrf52840")]
define_i2c_drivers!(
    SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0 => TWISPI0,
    SPIM1_SPIS1_TWIM1_TWIS1_SPI1_TWI1 => TWISPI1,
);
#[cfg(context = "nrf5340")]
define_i2c_drivers!(
    SERIAL0 => SERIAL0,
    SERIAL1 => SERIAL1,
);
