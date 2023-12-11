#![no_std]

use nrf52;

use riot_rs_rt::debug::println;

pub fn init() {
    println!("particle_xenon::init()");
    nrf52::init();
}
