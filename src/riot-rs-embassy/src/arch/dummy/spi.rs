use crate::arch;

/// Peripheral-agnostic SPI driver implementing [`embedded_hal_async::spi::SpiBus`].
///
/// The driver instance is meant to be shared between tasks using
/// [`SpiDevice`](crate::spi::SpiDevice).
///
/// This driver is not architecture-agnostic however, and has as many variants as the MCU has
/// SPI peripherals, each variant's name being the name of that peripheral.
/// Each enum variant has an associated value, which is the associated peripheral-specific driver
/// instance.
/// The names of peripheral-specific SPI drivers are `Spi$peripheral`, where `$peripheral` is the
/// name of the SPI peripheral (e.g, driver names can be `SpiSPI2`, `SpiSPI0`, or `SpiSERIAL2`).
/// The constructors of these peripheral-specific drivers depend on the architecture, please see
/// the examples and tests for reference.
pub enum Spi {}

pub fn init(peripherals: &mut arch::OptionalPeripherals) {
    unimplemented!();
}
