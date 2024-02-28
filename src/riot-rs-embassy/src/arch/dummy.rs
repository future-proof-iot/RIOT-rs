//! Dummy module used to satisfy platform-independent tooling.

/// Dummy type.
///
/// See the `OptionalPeripherals` type of your Embassy architecture crate instead.
#[derive(Default)]
pub struct OptionalPeripherals;

/// Dummy type.
pub struct Peripherals;

impl From<Peripherals> for OptionalPeripherals {
    fn from(_peripherals: Peripherals) -> Self {
        Self {}
    }
}

mod executor {
    use embassy_executor::SpawnToken;

    pub struct Executor;

    impl Executor {
        pub const fn new() -> Self {
            Self {}
        }

        pub fn start(&self, _: super::SWI) {}

        pub fn spawner(&self) -> Spawner {
            Spawner {}
        }
    }

    pub struct Spawner {}

    impl Spawner {
        #[allow(clippy::result_unit_err)]
        pub fn spawn<S>(&self, _token: SpawnToken<S>) -> Result<(), ()> {
            Ok(())
        }
    }
}
pub use executor::{Executor, Spawner};

#[derive(Default)]
pub struct Config;

pub fn init(_config: Config) -> OptionalPeripherals {
    unimplemented!();
}

pub struct SWI;
