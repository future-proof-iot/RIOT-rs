#![no_std]

use riot_rs_debug::log::debug;

pub fn init() {
    debug!("dwm1001::init()");
    nrf52::init();
}
