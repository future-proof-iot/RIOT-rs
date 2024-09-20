//! Provides architecture-agnostic I2C-related types, for controller mode.

use embassy_time::Duration;

pub use embedded_hal::i2c::Operation;
pub use fugit::KilohertzU32 as Kilohertz;

/// Timeout value for I2C operations.
///
/// Architectures are allowed to timeout earlier.
pub const I2C_TIMEOUT: Duration = Duration::from_millis(100);

/// I2C bus frequency.
// FIXME: rename this to Bitrate, and use kbit/s instead?
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Frequency {
    /// Standard mode: 100 kHz.
    _100k,
    /// Fast mode: 400 kHz.
    _400k,
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_i2c_from_frequency {
    () => {
        impl From<riot_rs_embassy_common::i2c::controller::Frequency> for Frequency {
            fn from(freq: riot_rs_embassy_common::i2c::controller::Frequency) -> Self {
                match freq {
                    riot_rs_embassy_common::i2c::controller::Frequency::_100k => Frequency::_100k,
                    riot_rs_embassy_common::i2c::controller::Frequency::_400k => Frequency::_400k,
                }
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_i2c_from_frequency_up_to {
    () => {
        impl From<riot_rs_embassy_common::i2c::controller::Frequency> for Frequency {
            fn from(freq: riot_rs_embassy_common::i2c::controller::Frequency) -> Self {
                match freq {
                    riot_rs_embassy_common::i2c::controller::Frequency::_100k => {
                        Frequency::UpTo100k($crate::i2c::controller::Kilohertz::kHz(100))
                    }
                    riot_rs_embassy_common::i2c::controller::Frequency::_400k => {
                        Frequency::UpTo400k($crate::i2c::controller::Kilohertz::kHz(400))
                    }
                }
            }
        }
    };
}

/// An I2C error, for controller mode.
// FIXME: make this non_exhaustive?
// NOTE(eq): not deriving `Eq` here because it *could* semantically contain floats later.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// A protocol error occurred (e.g., the transaction was terminated earlier than expected).
    Bus,
    /// Bus arbitration was lost (e.g., because there are multiple controllers on the bus).
    ArbitrationLoss,
    /// No acknowledgement was received when expected.
    NoAcknowledge(NoAcknowledgeSource),
    /// Overrun of the receive buffer.
    Overrun,
    /// Timeout when attempting to use the bus; most likely the target device is not connected.
    Timeout,
    /// An other error occurred.
    Other,
}

impl embedded_hal::i2c::Error for Error {
    fn kind(&self) -> embedded_hal::i2c::ErrorKind {
        #[expect(clippy::enum_glob_use, reason = "local import only")]
        use embedded_hal::i2c::ErrorKind::*;

        match self {
            Self::Bus => Bus,
            Self::ArbitrationLoss => ArbitrationLoss,
            Self::NoAcknowledge(ack_source) => NoAcknowledge((*ack_source).into()),
            Self::Overrun => Overrun,
            Self::Timeout | Self::Other => Other,
        }
    }
}

/// Indicates what protocol step was not acknowledged by the target device.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NoAcknowledgeSource {
    /// The device did not acknowledge its address.
    Address,
    /// The device did not acknowledge the data.
    Data,
    /// The device did not acknowledge either its address or its data.
    Unknown,
}

impl From<NoAcknowledgeSource> for embedded_hal::i2c::NoAcknowledgeSource {
    fn from(src: NoAcknowledgeSource) -> Self {
        match src {
            NoAcknowledgeSource::Address => embedded_hal::i2c::NoAcknowledgeSource::Address,
            NoAcknowledgeSource::Data => embedded_hal::i2c::NoAcknowledgeSource::Data,
            NoAcknowledgeSource::Unknown => embedded_hal::i2c::NoAcknowledgeSource::Unknown,
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_async_i2c_for_driver_enum {
    ($driver_enum:ident, $( $peripheral:ident ),*) => {
        impl $crate::reexports::embedded_hal_async::i2c::I2c for $driver_enum {
            async fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
                match self {
                    $(
                        Self::$peripheral(i2c) => {
                            $crate::handle_i2c_timeout_res!(i2c, read, address, read)
                        }
                    )*
                }
            }

            async fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
                match self {
                    $(
                        Self::$peripheral(i2c) => {
                            $crate::handle_i2c_timeout_res!(i2c, write, address, write)
                        }
                    )*
                }
            }

            async fn write_read(
                &mut self,
                address: u8,
                write: &[u8],
                read: &mut [u8],
            ) -> Result<(), Self::Error> {
                match self {
                    $(
                        Self::$peripheral(i2c) => {
                            $crate::handle_i2c_timeout_res!(i2c, write_read, address, write, read)
                        }
                    )*
                }
            }

            async fn transaction(
                &mut self,
                address: u8,
                operations: &mut [$crate::i2c::controller::Operation<'_>],
            ) -> Result<(), Self::Error> {
                match self {
                    $(
                        Self::$peripheral(i2c) => {
                            $crate::handle_i2c_timeout_res!(i2c, transaction, address, operations)
                        }
                    )*
                }
            }
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! handle_i2c_timeout_res {
    ($i2c:ident, $op:ident, $address:ident, $( $param:ident ),+) => {{
        let res = $crate::reexports::embassy_futures::select::select(
            // Disambiguate between the trait methods and the direct methods.
            $crate::reexports::embedded_hal_async::i2c::I2c::$op(&mut $i2c.twim, $address, $( $param ),+),
            $crate::reexports::embassy_time::Timer::after($crate::i2c::controller::I2C_TIMEOUT),
        ).await;

        if let $crate::reexports::embassy_futures::select::Either::First(op) = res {
            // `from_error` is defined in each arch
            op.map_err(from_error)
        } else {
            Err($crate::i2c::controller::Error::NoAcknowledge($crate::i2c::controller::NoAcknowledgeSource::Unknown))
        }
    }}
}
