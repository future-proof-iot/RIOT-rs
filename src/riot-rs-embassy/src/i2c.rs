//! Provides support for the I2C communication bus.
#![deny(missing_docs)]

use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice as InnerI2cDevice;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::Duration;

use crate::arch;

/// Timeout value for I2C operations.
///
/// Architectures are allowed to timeout earlier.
pub const I2C_TIMEOUT: Duration = Duration::from_millis(100);

/// An I2C driver implementing [`embedded_hal_async::i2c::I2c`].
///
/// Needs to be provided with an MCU-specific I2C driver tied to a specific I2C peripheral,
/// obtainable from the [`arch::i2c`] module.
///
/// See [`embedded_hal::i2c`] to learn more about how to share the bus.
// TODO: do we actually need a CriticalSectionRawMutex here?
pub type I2cDevice = InnerI2cDevice<'static, CriticalSectionRawMutex, arch::i2c::I2c>;

macro_rules! handle_timeout_res {
    ($i2c:ident, $op:ident, $address:ident, $( $param:ident ),+) => {{
        let res = embassy_futures::select::select(
            $i2c.twim.$op($address, $( $param ),+),
            embassy_time::Timer::after($crate::i2c::I2C_TIMEOUT),
        ).await;

        if let embassy_futures::select::Either::First(op) = res {
            Ok(op?)
        } else {
            Err($crate::i2c::Error::NoAcknowledge($crate::i2c::NoAcknowledgeSource::Unknown))
        }
    }}
}
pub(crate) use handle_timeout_res;

#[allow(unused_macros, reason = "used by arch modules")]
macro_rules! impl_async_i2c_for_driver_enum {
    ($driver_enum:ident, $( $peripheral:ident ),*) => {
        impl embedded_hal_async::i2c::I2c for $driver_enum {
            async fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
                match self {
                    $(
                        Self::$peripheral(i2c) => {
                            $crate::i2c::handle_timeout_res!(i2c, read, address, read)
                        }
                    )*
                }
            }

            async fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
                match self {
                    $(
                        Self::$peripheral(i2c) => {
                            $crate::i2c::handle_timeout_res!(i2c, write, address, write)
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
                            $crate::i2c::handle_timeout_res!(i2c, write_read, address, write, read)
                        }
                    )*
                }
            }

            async fn transaction(
                &mut self,
                address: u8,
                operations: &mut [Operation<'_>],
            ) -> Result<(), Self::Error> {
                match self {
                    $(
                        Self::$peripheral(i2c) => {
                            $crate::i2c::handle_timeout_res!(i2c, transaction, address, operations)
                        }
                    )*
                }
            }
        }
    }
}
#[allow(unused_imports, reason = "used by arch modules")]
pub(crate) use impl_async_i2c_for_driver_enum;

/// An I2C error.
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
        use embedded_hal::i2c::ErrorKind::*;

        match self {
            Self::Bus => Bus,
            Self::ArbitrationLoss => ArbitrationLoss,
            Self::NoAcknowledge(ack_source) => NoAcknowledge((*ack_source).into()),
            Self::Overrun => Overrun,
            Self::Timeout => Other, // FIXME: not sure, is this always a lack of ack?
            Self::Other => Other,
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

/// I2C bus frequency.
// FIXME: rename this to Bitrate, and use kbit/s instead?
pub enum Frequency {
    /// MCU-specific frequency.
    Arch(arch::i2c::Frequency),
    /// Standard mode: 100 kHz.
    _100k,
    /// Fast mode: 400 kHz.
    _400k,
}

impl From<crate::i2c::Frequency> for arch::i2c::Frequency {
    fn from(freq: crate::i2c::Frequency) -> Self {
        match freq {
            crate::i2c::Frequency::Arch(freq) => freq,
            crate::i2c::Frequency::_100k => arch::i2c::Frequency::_100k,
            crate::i2c::Frequency::_400k => arch::i2c::Frequency::_400k,
        }
    }
}
