#![no_std]

use ariel_os_debug::log::debug;

pub fn init() {
    debug!("nrf5340dk::init()");
    nrf5340::init();
}
