//! Provides support for the SPI communication bus.
#![deny(missing_docs)]

use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice as InnerSpiDevice;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

use crate::{arch, gpio};

/// An SPI driver implementing [`embedded_hal_async::spi::SpiDevice`].
///
/// Needs to be provided with an MCU-specific SPI driver tied to a specific SPI peripheral,
/// obtainable from the [`arch::spi`] module.
/// It also requires a [`gpio::Output`] for the chip-select (CS) signal.
///
/// See [`embedded_hal::spi`] to learn more about the distinction between an
/// [`SpiBus`](embedded_hal::spi::SpiBus) and an
/// [`SpiDevice`](embedded_hal::spi::SpiDevice).
// TODO: do we actually need a CriticalSectionRawMutex here?
pub type SpiDevice = InnerSpiDevice<'static, CriticalSectionRawMutex, arch::spi::Spi, gpio::Output>;

#[allow(unused_macros, reason = "used by arch modules")]
macro_rules! impl_async_spibus_for_driver_enum {
    ($driver_enum:ident, $( $peripheral:ident ),*) => {
        // The `SpiBus` trait represents exclusive ownership over the whole bus.
        impl embedded_hal_async::spi::SpiBus for $driver_enum {
            async fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
                match self {
                    $( Self::$peripheral(spi) => spi.spim.read(words).await, )*
                }
            }

            async fn write(&mut self, data: &[u8]) -> Result<(), Self::Error> {
                match self {
                    $( Self::$peripheral(spi) => spi.spim.write(data).await, )*
                }
            }

            async fn transfer(&mut self, rx: &mut [u8], tx: &[u8]) -> Result<(), Self::Error> {
                match self {
                    $( Self::$peripheral(spi) => spi.spim.transfer(rx, tx).await, )*
                }
            }

            async fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
                match self {
                    $( Self::$peripheral(spi) => spi.spim.transfer_in_place(words).await, )*
                }
            }

            async fn flush(&mut self) -> Result<(), Self::Error> {
                use embedded_hal_async::spi::SpiBus;
                match self {
                    $( Self::$peripheral(spi) => SpiBus::<u8>::flush(&mut spi.spim).await, )*
                }
            }
        }
    }
}
#[allow(unused_imports, reason = "used by arch modules")]
pub(crate) use impl_async_spibus_for_driver_enum;

// FIXME: rename this to Bitrate and use bps instead?
/// SPI bus frequency.
#[derive(Copy, Clone)]
pub enum Frequency {
    /// MCU-specific frequency.
    Arch(arch::spi::Frequency),
    /// 125 kHz.
    _125k,
    /// 250 kHz.
    _250k,
    /// 500 kHz.
    _500k,
    /// 1 MHz.
    _1M,
    /// 2 MHz.
    _2M,
    /// 4 MHz.
    _4M,
    /// 8 MHz.
    _8M,
}

impl From<Frequency> for arch::spi::Frequency {
    fn from(freq: Frequency) -> Self {
        match freq {
            Frequency::Arch(freq) => freq,
            Frequency::_125k => arch::spi::Frequency::_125k,
            Frequency::_250k => arch::spi::Frequency::_250k,
            Frequency::_500k => arch::spi::Frequency::_500k,
            Frequency::_1M => arch::spi::Frequency::_1M,
            Frequency::_2M => arch::spi::Frequency::_2M,
            Frequency::_4M => arch::spi::Frequency::_4M,
            Frequency::_8M => arch::spi::Frequency::_8M,
        }
    }
}

/// SPI mode.
///
/// - CPOL: Clock polarity.
/// - CPHA: Clock phase.
///
/// See the [Wikipedia page for details](https://en.wikipedia.org/wiki/Serial_Peripheral_Interface#Mode_numbers).
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Mode {
    /// CPOL = 0, CPHA = 0.
    Mode0,
    /// CPOL = 0, CPHA = 1.
    Mode1,
    /// CPOL = 1, CPHA = 0.
    Mode2,
    /// CPOL = 1, CPHA = 1.
    Mode3,
}

// FIXME: should we offer configuring the bit order? (hiding from the docs for now)
/// Order in which bits are transmitted.
///
/// Note: configuring the bit order is not supported on all architectures.
// NOTE(arch): the RP2040 and RP2350 always send the MSb first
#[doc(hidden)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BitOrder {
    /// Most significant bit first.
    MsbFirst,
    /// Least significant bit first.
    LsbFirst,
}

impl Default for BitOrder {
    fn default() -> Self {
        Self::MsbFirst
    }
}
