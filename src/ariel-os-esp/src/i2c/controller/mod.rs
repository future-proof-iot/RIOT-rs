//! Provides support for the I2C communication bus in controller mode.

use ariel_os_embassy_common::impl_async_i2c_for_driver_enum;
use esp_hal::{
    gpio::interconnect::PeripheralOutput, i2c::master::I2c as EspI2c, peripheral::Peripheral,
    peripherals, Async,
};

/// I2C bus configuration.
#[non_exhaustive]
#[derive(Clone)]
pub struct Config {
    /// The frequency at which the bus should operate.
    pub frequency: Frequency,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frequency: Frequency::_100k,
        }
    }
}

/// I2C bus frequency.
// NOTE(hal): the technical references only mention these frequencies.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Frequency {
    /// Standard mode.
    _100k,
    /// Fast mode.
    _400k,
}

#[doc(hidden)]
impl Frequency {
    pub const fn first() -> Self {
        Self::_100k
    }

    pub const fn last() -> Self {
        Self::_400k
    }

    pub const fn next(self) -> Option<Self> {
        match self {
            Self::_100k => Some(Self::_400k),
            Self::_400k => None,
        }
    }

    pub const fn prev(self) -> Option<Self> {
        match self {
            Self::_100k => None,
            Self::_400k => Some(Self::_100k),
        }
    }

    pub const fn khz(self) -> u32 {
        match self {
            Self::_100k => 100,
            Self::_400k => 400,
        }
    }
}

ariel_os_embassy_common::impl_i2c_from_frequency!();

impl From<Frequency> for fugit::HertzU32 {
    fn from(freq: Frequency) -> Self {
        match freq {
            Frequency::_100k => fugit::Rate::<u32, 1, 1>::kHz(100),
            Frequency::_400k => fugit::Rate::<u32, 1, 1>::kHz(400),
        }
    }
}

macro_rules! define_i2c_drivers {
    ($( $peripheral:ident ),* $(,)?) => {
        $(
            /// Peripheral-specific I2C driver.
            pub struct $peripheral {
                twim: EspI2c<'static, Async>,
            }

            impl $peripheral {
                /// Returns a driver implementing [`embedded_hal_async::i2c::I2c`] for this
                /// I2C peripheral.
                #[expect(clippy::new_ret_no_self)]
                #[must_use]
                pub fn new<SDA: PeripheralOutput, SCL: PeripheralOutput>(
                    sda_pin: impl Peripheral<P = SDA> + 'static,
                    scl_pin: impl Peripheral<P = SCL> + 'static,
                    config: Config,
                ) -> I2c {
                    let mut twim_config = esp_hal::i2c::master::Config::default();
                    twim_config.frequency = config.frequency.into();
                    // Disable timeout as we implement it at a higher level.
                    twim_config.timeout = None;

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

                    let twim = EspI2c::new(
                        i2c_peripheral,
                        twim_config,
                    )
                        .into_async()
                        .with_sda(sda_pin)
                        .with_scl(scl_pin);

                    I2c::$peripheral(Self { twim })
                }
            }
        )*

        /// Peripheral-agnostic driver.
        pub enum I2c {
            $(
                #[doc = concat!(stringify!($peripheral), " peripheral.")]
                $peripheral($peripheral),
            )*
        }

        impl embedded_hal_async::i2c::ErrorType for I2c {
            type Error = ariel_os_embassy_common::i2c::controller::Error;
        }

        impl_async_i2c_for_driver_enum!(I2c, $( $peripheral ),*);
    }
}

// We cannot impl From because both types are external to this crate.
fn from_error(err: esp_hal::i2c::master::Error) -> ariel_os_embassy_common::i2c::controller::Error {
    use esp_hal::i2c::master::Error::*;

    use ariel_os_embassy_common::i2c::controller::{Error, NoAcknowledgeSource};

    match err {
        ExceedingFifo => Error::Overrun,
        AckCheckFailed => Error::NoAcknowledge(NoAcknowledgeSource::Unknown),
        TimeOut => Error::Timeout,
        ArbitrationLost => Error::ArbitrationLoss,
        ExecIncomplete => Error::Other,
        CommandNrExceeded => Error::Other,
        InvalidZeroLength => Error::Other,
    }
}

// FIXME: support other MCUs
// Define a driver per peripheral
#[cfg(context = "esp32c6")]
define_i2c_drivers!(I2C0);
