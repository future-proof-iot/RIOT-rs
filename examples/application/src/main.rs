#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use riot_rs::embassy::{arch, Application, ApplicationError, Drivers};

use riot_rs::rt::debug::println;

struct MyApplication;

impl Application for MyApplication {
    fn init() -> &'static dyn Application {
        &Self {}
    }

    fn start(
        &self,
        _peripherals: &mut arch::OptionalPeripherals,
        spawner: embassy_executor::Spawner,
        _drivers: Drivers,
    ) -> Result<(), ApplicationError> {
        spawner.spawn(task()).unwrap();
        Ok(())
    }
}

#[embassy_executor::task]
async fn task() {
    println!("MyApplication::start()");
    // ...
}

riot_rs::embassy::riot_initialize!(MyApplication);
