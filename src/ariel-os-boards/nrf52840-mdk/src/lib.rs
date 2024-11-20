#![no_std]

use riot_rs_debug::log::debug;

pub fn init() {
    debug!("nrf52840-mdk::init()");
    nrf52::init();
}
