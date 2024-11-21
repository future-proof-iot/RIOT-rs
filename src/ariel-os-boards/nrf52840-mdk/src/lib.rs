#![no_std]

use ariel_os_debug::log::debug;

pub fn init() {
    debug!("nrf52840-mdk::init()");
    nrf52::init();
}
