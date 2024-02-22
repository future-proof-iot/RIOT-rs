#![no_std]

use riot_rs_rt::debug::println;

pub fn init() {
    println!("microbit::init()");
    nrf51::init();
}
