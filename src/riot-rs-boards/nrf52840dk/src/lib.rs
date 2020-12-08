#![no_std]

use nrf52;

use riot_rs_rt::debug::println;

pub fn init() {
    println!("nrf52840dk::init()");
    nrf52::init();
}
