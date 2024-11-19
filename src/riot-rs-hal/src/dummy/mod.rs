//! Dummy module used to satisfy platform-independent tooling.
// TODO: redirect to the manufacturer-specific crate documentation when we publish it, and
// mark every item in this dummy module `doc(hidden)`

mod executor;
pub mod gpio;

pub mod peripheral {
    pub use embassy_hal_internal::Peripheral;
}

#[cfg(feature = "hwrng")]
pub mod hwrng;

#[cfg(feature = "i2c")]
pub mod i2c;

pub mod identity {
    use riot_rs_embassy_common::identity;

    pub type DeviceId = identity::NoDeviceId<identity::NotImplemented>;
}

#[cfg(feature = "spi")]
pub mod spi;

#[cfg(feature = "storage")]
pub mod storage;

#[cfg(feature = "usb")]
pub mod usb;

pub use executor::{Executor, Spawner};

/// Dummy type.
///
/// See the `OptionalPeripherals` type of your Embassy architecture crate instead.
pub struct OptionalPeripherals;

/// Dummy type.
pub struct Peripherals;

impl From<Peripherals> for OptionalPeripherals {
    fn from(_peripherals: Peripherals) -> Self {
        Self {}
    }
}

pub fn init() -> OptionalPeripherals {
    unimplemented!();
}

pub struct SWI;
