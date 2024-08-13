use embassy_rp::{
    bind_interrupts,
    i2c::{InterruptHandler, SclPin, SdaPin},
    peripherals, Peripheral,
};
use embedded_hal_async::i2c::Operation;

use crate::{arch, i2c::impl_async_i2c_for_driver_enum};

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

pub fn init(peripherals: &mut arch::OptionalPeripherals) {
    // Take all I2C peripherals and do nothing with them.
    cfg_if::cfg_if! {
        if #[cfg(context = "rp2040")] {
            let _ = peripherals.I2C0.take().unwrap();
            let _ = peripherals.I2C1.take().unwrap();
        } else {
            compile_error!("this RP chip is not supported");
        }
    }
}

macro_rules! define_i2c_drivers {
    ($( $interrupt:ident => $peripheral:ident ),* $(,)?) => {
        $(
            /// Peripheral-specific I2C driver.
            pub struct $peripheral {
                twim: embassy_rp::i2c::I2c<'static, peripherals::$peripheral, embassy_rp::i2c::Async>,
            }

            impl $peripheral {
                #[must_use]
                pub fn new(
                    sda_pin: impl Peripheral<P: SdaPin<peripherals::$peripheral>> + 'static,
                    scl_pin: impl Peripheral<P: SclPin<peripherals::$peripheral>> + 'static,
                    config: Config,
                ) -> I2c {
                    let mut i2c_config = embassy_rp::i2c::Config::default();
                    i2c_config.frequency = config.frequency as u32;

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
                    let i2c_peripheral = unsafe { peripherals::$peripheral::steal() };

                    let i2c = embassy_rp::i2c::I2c::new_async(
                        i2c_peripheral,
                        scl_pin,
                        sda_pin,
                        Irqs,
                        i2c_config,
                    );

                    I2c::$peripheral(Self { twim: i2c })
                }
            }
        )*

        /// Peripheral-agnostic driver.
        pub enum I2c {
            $( $peripheral($peripheral), )*
        }

        impl embedded_hal_async::i2c::ErrorType for I2c {
            type Error = embassy_rp::i2c::Error;
        }

        impl_async_i2c_for_driver_enum!(I2c, $( $peripheral ),*);
    }
}

// Define a driver per peripheral
define_i2c_drivers!(
    I2C0_IRQ => I2C0,
    I2C1_IRQ => I2C1,
);
