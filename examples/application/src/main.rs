#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use riot_rs::embassy::{arch, Application, ApplicationInitError};

use riot_rs::rt::debug::println;

struct MyApplication {
    state: u32, // some state
}

impl Application for MyApplication {
    fn initialize(
        _peripherals: &mut arch::OptionalPeripherals,
    ) -> Result<&dyn Application, ApplicationInitError> {
        println!("MyApplication::initialize()");
        Ok(&Self { state: 0 })
    }

    fn start(&self, _spawner: embassy_executor::Spawner) {
        println!("MyApplication::start()");
        // ...
    }
}

riot_rs::embassy::riot_initialize!(MyApplication);
