//! Architecture- and MCU-specific types for I2C.
//!
//! This module provides a driver for each I2C peripheral, the driver name being the same as the
//! peripheral; see the tests and examples to learn how to instantiate them.
//! These driver instances are meant to be shared between tasks using
//! [`I2cDevice`](crate::i2c::I2cDevice).

use crate::arch;

/// Peripheral-agnostic I2C driver implementing [`embedded_hal_async::i2c::I2c`].
///
/// This type is not meant to be instantiated directly; instead instantiate a peripheral-specific
/// driver provided by this module.
// NOTE: we keep this type public because it may still required in user-written type signatures.
pub enum I2c {
    // Make the docs show that this enum has variants, but do not show any because they are
    // MCU-specific.
    #[doc(hidden)]
    Hidden,
}

pub(crate) fn init(peripherals: &mut arch::OptionalPeripherals) {
    unimplemented!();
}
