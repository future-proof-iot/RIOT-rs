#![no_std]

use ariel_os_debug::log::debug;

pub fn init() {
    debug!("microbit::init()");
    nrf51::init();
}
