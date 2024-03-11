#![no_std]

use riot_rs_debug::println;

pub fn init() {
    println!("dwm1001::init()");
    nrf52::init();
}
