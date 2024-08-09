use embedded_hal_async::i2c::Operation;
use esp_hal::{
    gpio::{InputPin, OutputPin},
    i2c::I2C,
    peripheral::Peripheral,
    peripherals, Async,
};

use crate::{arch, i2c::impl_async_i2c_for_driver_enum};

#[non_exhaustive]
#[derive(Clone)]
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

// FIXME: check how well this matches the ESP32 capabilities
// TODO: allow more free-from values?
#[derive(Copy, Clone)]
#[repr(u32)]
pub enum Frequency {
    K100 = 100_000,
    K250 = 250_000,
    K400 = 400_000,
    M1 = 1_000_000,
}

impl From<Frequency> for fugit::HertzU32 {
    fn from(freq: Frequency) -> Self {
        match freq {
            Frequency::K100 => fugit::Rate::<u32, 1, 1>::kHz(100),
            Frequency::K250 => fugit::Rate::<u32, 1, 1>::kHz(250),
            Frequency::K400 => fugit::Rate::<u32, 1, 1>::kHz(400),
            Frequency::M1 => fugit::Rate::<u32, 1, 1>::MHz(1),
        }
    }
}

macro_rules! define_i2c_drivers {
    ($( $peripheral:ident ),* $(,)?) => {
        // paste allows to create new identifiers by concatenation using `[<foo bar>]`.
        paste::paste! {
            $(
                /// Peripheral-specific I2C driver.
                pub struct [<I2c $peripheral>] {
                    twim: I2C<'static, peripherals::$peripheral, Async>,
                }

                impl [<I2c $peripheral>] {
                    #[must_use]
                    pub fn new<SDA, SCL>(
                        i2c_peripheral: impl Peripheral<P = peripherals::$peripheral> + 'static,
                        sda_pin: impl Peripheral<P = SDA> + 'static,
                        scl_pin: impl Peripheral<P = SCL> + 'static,
                        config: Config,
                    ) -> Self
                        where SDA: OutputPin + InputPin,
                              SCL: OutputPin + InputPin,
                    {
                        let frequency = config.frequency.into();
                        let clocks = arch::CLOCKS.get().unwrap();

                        // FIXME: use `new_with_timeout_async()` instead?
                        let twim = I2C::new_async(i2c_peripheral, sda_pin, scl_pin, frequency, &clocks);

                        Self { twim }
                    }
                }
            )*

            /// Peripheral-agnostic driver.
            pub enum I2c {
                $( $peripheral([<I2c $peripheral>]), )*
            }

            impl embedded_hal_async::i2c::ErrorType for I2c {
                type Error = esp_hal::i2c::Error;
            }

            impl_async_i2c_for_driver_enum!(I2c, $( $peripheral ),*);
        }
    }
}

// FIXME: support other archs
// Define a driver per peripheral
#[cfg(context = "esp32c6")]
define_i2c_drivers!(I2C0);
