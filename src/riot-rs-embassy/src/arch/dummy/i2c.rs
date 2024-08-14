use crate::arch;

/// Peripheral-agnostic I2C driver implementing [`embedded_hal_async::i2c::I2c`].
///
/// The driver instance is meant to be shared between tasks using
/// [`I2cDevice`](crate::i2c::I2cDevice).
///
/// This driver is not architecture-agnostic however, and has as many variants as the MCU has
/// I2C peripherals, each variant's name being the name of that peripheral.
/// Each enum variant has an associated value, which is the associated peripheral-specific driver
/// instance.
/// The names of peripheral-specific I2C drivers are `I2c$peripheral`, where `$peripheral` is the
/// name of the I2C peripheral (e.g, driver names can be `I2cI2C0`, `I2cTWISPI0`, or `I2cSERIAL0`).
/// The constructors of these peripheral-specific drivers depend on the architecture, please see
/// the examples and tests for reference.
pub enum I2c {}

pub fn init(peripherals: &mut arch::OptionalPeripherals) {
    unimplemented!();
}
