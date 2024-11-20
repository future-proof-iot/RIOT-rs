#![no_main]
#![feature(used_with_arg)]

// As the macro will fail, this import will not get used
#[allow(unused_imports)]
use ariel_os::asynch::Spawner;

// FAIL: the `peripherals` parameter is required in this case
#[ariel_os::spawner(autostart)]
fn main(_spawner: Spawner, _peripherals: Peripherals) {}

struct Peripherals;
