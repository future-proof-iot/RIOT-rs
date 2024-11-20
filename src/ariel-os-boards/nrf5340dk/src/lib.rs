#![no_std]

use riot_rs_debug::log::debug;

pub fn init() {
    debug!("nrf5340dk::init()");
    nrf5340::init();
}
