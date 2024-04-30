//! Dummy module used to satisfy platform-independent tooling.

mod executor;
pub mod gpio;

#[cfg(feature = "hwrng")]
pub mod hwrng;

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

#[derive(Default)]
pub struct Config;

#[allow(clippy::needless_pass_by_value, reason = "dummy implementation")]
#[must_use]
pub fn init(_config: Config) -> OptionalPeripherals {
    unimplemented!();
}

pub struct SWI;
