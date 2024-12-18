//! Dummy module used to satisfy platform-independent tooling.

#![allow(
    clippy::missing_errors_doc,
    reason = "this module's items are hidden in the docs"
)]
#![allow(
    clippy::module_name_repetitions,
    reason = "this dummy module mimics manufacturer-specific crates"
)]
#![allow(
    clippy::needless_pass_by_value,
    reason = "this dummy module mimics manufacturer-specific crates"
)]

mod executor;

#[doc(hidden)]
pub mod gpio;

#[doc(hidden)]
pub mod peripheral {
    pub use embassy_hal_internal::Peripheral;
}

#[doc(hidden)]
#[cfg(feature = "hwrng")]
pub mod hwrng;

#[doc(hidden)]
#[cfg(feature = "i2c")]
pub mod i2c;

#[doc(hidden)]
pub mod identity {
    use ariel_os_embassy_common::identity;

    pub type DeviceId = identity::NoDeviceId<identity::NotImplemented>;
}

#[doc(hidden)]
#[cfg(feature = "spi")]
pub mod spi;

#[doc(hidden)]
#[cfg(feature = "storage")]
pub mod storage;

#[doc(hidden)]
#[cfg(feature = "usb")]
pub mod usb;

pub use executor::{Executor, Spawner};

#[doc(hidden)]
/// Dummy type.
///
/// See the `OptionalPeripherals` type of your Embassy HAL crate instead.
pub struct OptionalPeripherals;

#[doc(hidden)]
/// Dummy type.
pub struct Peripherals;

impl From<Peripherals> for OptionalPeripherals {
    fn from(_peripherals: Peripherals) -> Self {
        Self {}
    }
}

#[doc(hidden)]
#[must_use]
pub fn init() -> OptionalPeripherals {
    unimplemented!();
}

#[doc(hidden)]
pub struct SWI;
