#![no_std]

use ariel_os_debug::log::debug;

pub fn init() {
    debug!("nrf52840dk::init()");
    nrf52::init();
}
