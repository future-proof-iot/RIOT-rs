#![no_std]

use riot_rs_debug::println;

pub fn init() {
    println!("microbit::init()");
    nrf51::init();
}
