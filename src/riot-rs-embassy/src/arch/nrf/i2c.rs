use embassy_nrf::{
    bind_interrupts,
    gpio::Pin as GpioPin,
    peripherals,
    twim::{InterruptHandler, Twim},
    Peripheral,
};
use embedded_hal_async::i2c::Operation;

use crate::{arch, i2c::impl_async_i2c_for_driver_enum};

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

pub fn init(peripherals: &mut arch::OptionalPeripherals) {
    // Take all I2C peripherals and do nothing with them.
    cfg_if::cfg_if! {
        if #[cfg(context = "nrf52840")] {
            let _ = peripherals.TWISPI0.take().unwrap();
            let _ = peripherals.TWISPI1.take().unwrap();
        } else if #[cfg(context = "nrf5340")] {
            let _ = peripherals.SERIAL0.take().unwrap();
            let _ = peripherals.SERIAL1.take().unwrap();
        } else {
            compile_error!("this nRF chip is not supported");
        }
    }
}

macro_rules! define_i2c_drivers {
    ($( $interrupt:ident => $peripheral:ident ),* $(,)?) => {
        $(
            /// Peripheral-specific I2C driver.
            pub struct $peripheral {
                twim: Twim<'static, peripherals::$peripheral>,
            }

            impl $peripheral {
                #[must_use]
                pub fn new(
                    sda_pin: impl Peripheral<P: GpioPin> + 'static,
                    scl_pin: impl Peripheral<P: GpioPin> + 'static,
                    config: Config,
                ) -> I2c {
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

                    // Make this struct a compile-time-enforced singleton: having multiple statics
                    // defined with the same name would result in a compile-time error.
                    paste::paste! {
                        #[allow(dead_code)]
                        static [<PREVENT_MULTIPLE_ $peripheral>]: () = ();
                    }

                    // FIXME(safety): enforce that the init code indeed has run
                    // SAFETY: this struct being a singleton prevents us from stealing the
                    // peripheral multiple times.
                    let twim_peripheral = unsafe { peripherals::$peripheral::steal() };

                    let twim = Twim::new(twim_peripheral, Irqs, sda_pin, scl_pin, twim_config);

                    I2c::$peripheral(Self { twim })
                }
            }
        )*

        /// Peripheral-agnostic driver.
        pub enum I2c {
            $( $peripheral($peripheral), )*
        }

        impl embedded_hal_async::i2c::ErrorType for I2c {
            type Error = embassy_nrf::twim::Error;
        }

        impl_async_i2c_for_driver_enum!(I2c, $( $peripheral ),*);
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
