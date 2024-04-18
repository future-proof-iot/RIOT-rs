//! Dummy module used to satisfy platform-independent tooling.

mod executor;

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

pub fn init(_config: Config) -> OptionalPeripherals {
    unimplemented!();
}

pub struct SWI;
