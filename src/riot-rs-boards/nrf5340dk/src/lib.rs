#![no_std]

use riot_rs_debug::println;

pub fn init() {
    println!("nrf5340dk::init()");
    nrf5340::init();
}
