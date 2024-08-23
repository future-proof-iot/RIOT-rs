//! Architecture- and MCU-specific types for SPI.
//!
//! This module provides a driver for each SPI peripheral, the driver name being the same as the
//! peripheral; see the tests and examples to learn how to instantiate them.
//! These driver instances are meant to be shared between tasks using
//! [`SpiDevice`](crate::spi::SpiDevice).

use crate::arch;

/// Peripheral-agnostic SPI driver implementing [`embedded_hal_async::spi::SpiBus`].
///
/// This type is not meant to be instantiated directly; instead instantiate a peripheral-specific
/// driver provided by this module.
// NOTE: we keep this type public because it may still required in user-written type signatures.
pub enum Spi {
    // Make the docs show that this enum has variants, but do not show any because they are
    // MCU-specific.
    #[doc(hidden)]
    Hidden,
}

pub(crate) fn init(peripherals: &mut arch::OptionalPeripherals) {
    unimplemented!();
}
