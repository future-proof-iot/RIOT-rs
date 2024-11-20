#![no_std]

use riot_rs_debug::log::debug;

pub fn init() {
    debug!("particle_xenon::init()");
    nrf52::init();
}
