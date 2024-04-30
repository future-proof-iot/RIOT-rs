use embassy_executor::SpawnToken;

use crate::arch;

pub struct Executor;

impl Executor {
    #[allow(clippy::new_without_default)]
    #[must_use]
    pub const fn new() -> Self {
        // Actually return a value instead of marking it unimplemented like other dummy
        // functions, because this function is const and is thus run during compilation
        Self {}
    }

    pub fn start(&self, _: arch::SWI) {
        unimplemented!();
    }

    #[must_use]
    pub fn spawner(&self) -> Spawner {
        unimplemented!();
    }
}

pub struct Spawner;

impl Spawner {
    #[allow(
        clippy::result_unit_err,
        clippy::needless_pass_by_value,
        clippy::missing_errors_doc,
        reason = "dummy implementation"
    )]
    pub fn spawn<S>(&self, _token: SpawnToken<S>) -> Result<(), ()> {
        unimplemented!();
    }
    #[allow(clippy::needless_pass_by_value, reason = "dummy implementation")]
    pub fn must_spawn<S>(&self, _token: SpawnToken<S>) {}
}
