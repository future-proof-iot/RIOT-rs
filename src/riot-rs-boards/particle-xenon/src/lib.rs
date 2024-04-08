#![no_std]

use riot_rs_debug::println;

pub fn init() {
    println!("particle_xenon::init()");
    nrf52::init();
}
