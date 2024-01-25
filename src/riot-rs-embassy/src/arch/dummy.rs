//! Dummy module used to satisfy platform-independent tooling.

/// Dummy type.
///
/// See the `OptionalPeripherals` type of your Embassy architecture crate instead.
pub struct OptionalPeripherals;

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
        pub fn spawn<S>(&self, token: SpawnToken<S>) -> Result<(), ()> {
            Ok(())
        }
    }
}
pub use executor::{Executor, Spawner};

pub fn init(_: ()) -> OptionalPeripherals {
    OptionalPeripherals {}
}

pub struct SWI;
