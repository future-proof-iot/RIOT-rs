#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use riot_rs::embassy::{arch, Application, ApplicationInitError, Drivers, InitializationArgs};

use riot_rs::rt::debug::println;

struct MyApplication {
    state: u32, // some state
}

impl Application for MyApplication {
    fn initialize(
        _peripherals: &mut arch::OptionalPeripherals,
        _init_args: InitializationArgs,
    ) -> Result<&dyn Application, ApplicationInitError> {
        println!("MyApplication::initialize()");
        Ok(&Self { state: 0 })
    }

    fn start(&self, _spawner: embassy_executor::Spawner, _drivers: Drivers) {
        println!("MyApplication::start()");
        // ...
    }
}

riot_rs::embassy::riot_initialize!(MyApplication);

#[no_mangle]
fn riot_main() {
    riot_rs::rt::debug::exit(Ok(()))
}
