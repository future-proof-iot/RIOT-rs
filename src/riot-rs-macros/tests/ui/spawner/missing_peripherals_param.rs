#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

// As the macro will fail, this import will not get used
#[allow(unused_imports)]
use riot_rs::Spawner;

// FAIL: the `peripherals` parameter is required in this case
#[riot_rs::spawner(autostart)]
fn main(_spawner: Spawner, _peripherals: Peripherals) {}

struct Peripherals;
