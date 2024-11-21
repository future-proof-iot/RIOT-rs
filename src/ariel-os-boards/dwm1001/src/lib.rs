#![no_std]

use ariel_os_debug::log::debug;

pub fn init() {
    debug!("dwm1001::init()");
    nrf52::init();
}
