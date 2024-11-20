#![no_std]

use riot_rs_debug::log::debug;

pub fn init() {
    debug!("microbit::init()");
    nrf51::init();
}
