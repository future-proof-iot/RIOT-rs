use embassy_executor::SpawnToken;

pub struct Executor;

impl Executor {
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        // Actually return a value instead of marking it unimplemented like other dummy
        // functions, because this function is const and is thus run during compilation
        Self {}
    }

    pub fn start(&self, _: crate::SWI) {
        unimplemented!();
    }

    pub fn spawner(&self) -> Spawner {
        unimplemented!();
    }
}

pub struct Spawner;

impl Spawner {
    #[allow(clippy::result_unit_err)]
    pub fn spawn<S>(&self, _token: SpawnToken<S>) -> Result<(), ()> {
        unimplemented!();
    }
    pub fn must_spawn<S>(&self, _token: SpawnToken<S>) {}
}
