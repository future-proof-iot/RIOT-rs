#![no_std]

use riot_rs_rt::debug::println;

pub fn init() {
    println!("nrf52840-mdk::init()");
    nrf52::init();
}
