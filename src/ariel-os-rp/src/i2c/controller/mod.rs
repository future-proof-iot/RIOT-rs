//! Provides support for the I2C communication bus in controller mode.

use embassy_rp::{
    bind_interrupts,
    i2c::{InterruptHandler, SclPin, SdaPin},
    peripherals, Peripheral,
};
use ariel_os_embassy_common::{i2c::controller::Kilohertz, impl_async_i2c_for_driver_enum};

const KHZ_TO_HZ: u32 = 1000;

/// I2C bus configuration.
// We do not provide configuration for internal pull-ups as the RP2040 datasheet mentions in
// section 4.3.1.3 that the GPIO used should have pull-ups enabled.
#[derive(Clone)]
#[non_exhaustive]
pub struct Config {
    /// The frequency at which the bus should operate.
    pub frequency: Frequency,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frequency: Frequency::UpTo100k(Kilohertz::kHz(100)),
        }
    }
}

/// I2C bus frequency.
// FIXME(embassy): fast mode plus is supported by hardware but requires additional configuration
// that Embassy does not seem to currently provide.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
pub enum Frequency {
    /// Standard mode.
    UpTo100k(Kilohertz), // FIXME: use a ranged integer?
    /// Fast mode.
    UpTo400k(Kilohertz), // FIXME: use a ranged integer?
}

#[doc(hidden)]
impl Frequency {
    pub const fn first() -> Self {
        Self::UpTo100k(Kilohertz::kHz(1))
    }

    pub const fn last() -> Self {
        Self::UpTo400k(Kilohertz::kHz(400))
    }

    pub const fn next(self) -> Option<Self> {
        match self {
            Self::UpTo100k(f) => {
                if f.to_kHz() < 100 {
                    // NOTE(no-overflow): `f` is small enough due to if condition
                    Some(Self::UpTo100k(Kilohertz::kHz(f.to_kHz() + 1)))
                } else {
                    Some(Self::UpTo400k(Kilohertz::kHz(self.khz() + 1)))
                }
            }
            Self::UpTo400k(f) => {
                if f.to_kHz() < 400 {
                    // NOTE(no-overflow): `f` is small enough due to if condition
                    Some(Self::UpTo400k(Kilohertz::kHz(f.to_kHz() + 1)))
                } else {
                    None
                }
            }
        }
    }

    pub const fn prev(self) -> Option<Self> {
        match self {
            Self::UpTo100k(f) => {
                if f.to_kHz() > 1 {
                    // NOTE(no-overflow): `f` is large enough due to if condition
                    Some(Self::UpTo100k(Kilohertz::kHz(f.to_kHz() - 1)))
                } else {
                    None
                }
            }
            Self::UpTo400k(f) => {
                if f.to_kHz() > 100 + 1 {
                    // NOTE(no-overflow): `f` is large enough due to if condition
                    Some(Self::UpTo400k(Kilohertz::kHz(f.to_kHz() - 1)))
                } else {
                    Some(Self::UpTo100k(Kilohertz::kHz(self.khz() - 1)))
                }
            }
        }
    }

    pub const fn khz(self) -> u32 {
        match self {
            Self::UpTo100k(f) | Self::UpTo400k(f) => f.to_kHz(),
        }
    }
}

ariel_os_embassy_common::impl_i2c_from_frequency_up_to!();

macro_rules! define_i2c_drivers {
    ($( $interrupt:ident => $peripheral:ident ),* $(,)?) => {
        $(
            /// Peripheral-specific I2C driver.
            pub struct $peripheral {
                twim: embassy_rp::i2c::I2c<'static, peripherals::$peripheral, embassy_rp::i2c::Async>,
            }

            impl $peripheral {
                /// Returns a driver implementing [`embedded_hal_async::i2c::I2c`] for this
                /// I2C peripheral.
                #[expect(clippy::new_ret_no_self)]
                #[must_use]
                pub fn new(
                    sda_pin: impl Peripheral<P: SdaPin<peripherals::$peripheral>> + 'static,
                    scl_pin: impl Peripheral<P: SclPin<peripherals::$peripheral>> + 'static,
                    config: Config,
                ) -> I2c {
                    let mut i2c_config = embassy_rp::i2c::Config::default();
                    i2c_config.frequency = config.frequency.khz() * KHZ_TO_HZ;

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

                    // NOTE(hal): even though we handle bus timeout at a higher level as well, it
                    // does not seem possible to disable the timeout feature on RP.
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
fn from_error(err: embassy_rp::i2c::Error) -> ariel_os_embassy_common::i2c::controller::Error {
    use embassy_rp::i2c::{AbortReason, Error::*};

    use ariel_os_embassy_common::i2c::controller::{Error, NoAcknowledgeSource};

    match err {
        Abort(reason) => match reason {
            AbortReason::NoAcknowledge => Error::NoAcknowledge(NoAcknowledgeSource::Unknown),
            AbortReason::ArbitrationLoss => Error::ArbitrationLoss,
            AbortReason::TxNotEmpty(_) => Error::Other,
            AbortReason::Other(_) => Error::Other,
        },
        InvalidReadBufferLength => Error::Other,
        InvalidWriteBufferLength => Error::Other,
        AddressOutOfRange(_) => Error::Other,
        AddressReserved(_) => Error::Other,
    }
}

// Define a driver per peripheral
define_i2c_drivers!(
    I2C0_IRQ => I2C0,
    I2C1_IRQ => I2C1,
);
