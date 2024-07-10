#![no_std]

use riot_rs_debug::log::debug;

pub fn init() {
    debug!("microbit_v2::init()");
    nrf52::init();
}
