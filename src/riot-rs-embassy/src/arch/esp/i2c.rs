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

pub fn init(peripherals: &mut arch::OptionalPeripherals) {
    // Take all I2C peripherals and do nothing with them.
    cfg_if::cfg_if! {
        if #[cfg(context = "esp32c6")] {
            let _ = peripherals.I2C0.take().unwrap();
        } else {
            compile_error!("this ESP32 chip is not supported");
        }
    }
}

macro_rules! define_i2c_drivers {
    ($( $peripheral:ident ),* $(,)?) => {
        $(
            /// Peripheral-specific I2C driver.
            pub struct $peripheral {
                twim: I2C<'static, peripherals::$peripheral, Async>,
            }

            impl $peripheral {
                #[must_use]
                pub fn new<SDA: OutputPin + InputPin, SCL: OutputPin + InputPin>(
                    sda_pin: impl Peripheral<P = SDA> + 'static,
                    scl_pin: impl Peripheral<P = SCL> + 'static,
                    config: Config,
                ) -> I2c {
                    let frequency = config.frequency.into();
                    let clocks = arch::CLOCKS.get().unwrap();

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

                    // NOTE(arch): even though we handle bus timeout at a higher level as well, it
                    // does not seem possible to disable the timeout feature on ESP; so we keep the
                    // default timeout instead (encoded as `None`).
                    let timeout = None;
                    let twim = I2C::new_with_timeout_async(
                        i2c_peripheral,
                        sda_pin,
                        scl_pin,
                        frequency,
                        &clocks,
                        timeout,
                    );

                    I2c::$peripheral(Self { twim })
                }
            }
        )*

        /// Peripheral-agnostic driver.
        pub enum I2c {
            $( $peripheral($peripheral), )*
        }

        impl embedded_hal_async::i2c::ErrorType for I2c {
            type Error = crate::i2c::Error;
        }

        impl_async_i2c_for_driver_enum!(I2c, $( $peripheral ),*);
    }
}

impl From<esp_hal::i2c::Error> for crate::i2c::Error {
    fn from(err: esp_hal::i2c::Error) -> Self {
        use esp_hal::i2c::Error::*;

        use crate::i2c::{Error, NoAcknowledgeSource};

        match err {
            ExceedingFifo => Error::Overrun,
            AckCheckFailed => Error::NoAcknowledge(NoAcknowledgeSource::Unknown),
            TimeOut => Error::Timeout,
            ArbitrationLost => Error::ArbitrationLoss,
            ExecIncomplete => Error::Other,
            CommandNrExceeded => Error::Other,
        }
    }
}

// FIXME: support other archs
// Define a driver per peripheral
#[cfg(context = "esp32c6")]
define_i2c_drivers!(I2C0);
