//! Provides architecture-agnostic SPI-related types.

#[doc(alias = "master")]
pub mod main;

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
