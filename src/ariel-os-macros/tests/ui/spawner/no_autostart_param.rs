#![no_main]
#![feature(used_with_arg)]

// As the macro will fail, this import will not get used
#[allow(unused_imports)]
use ariel_os::asynch::Spawner;

// FAIL: the macro expects a mandatory `autostart` parameter
#[ariel_os::spawner]
fn main(spawner: Spawner) {}
