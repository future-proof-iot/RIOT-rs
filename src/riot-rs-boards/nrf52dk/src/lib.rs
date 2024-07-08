#![no_std]

use riot_rs_debug::log::debug;

pub fn init() {
    debug!("nrf52dk::init()");
    nrf52::init();
}
