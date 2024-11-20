#![no_main]
#![feature(used_with_arg)]

// As the macro will fail, this import will not get used
#[allow(unused_imports)]
use ariel_os::asynch::Spawner;

// FAIL: spawner functions cannot be async
#[ariel_os::spawner(autostart)]
async fn main(spawner: Spawner) {}
