#![no_main]
#![feature(used_with_arg)]

// As the macro will fail, this import will not get used
#[allow(unused_imports)]
use riot_rs::asynch::Spawner;

// FAIL: the macro expects a mandatory `autostart` parameter
#[riot_rs::spawner]
fn main(spawner: Spawner) {}
