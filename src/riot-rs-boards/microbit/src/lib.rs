#![no_std]

use nrf51;

use riot_rs_rt::debug::println;

pub fn init() {
    println!("microbit::init()");
    nrf51::init();
}
