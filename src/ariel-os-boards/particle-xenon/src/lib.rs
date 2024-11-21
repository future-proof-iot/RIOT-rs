#![no_std]

use ariel_os_debug::log::debug;

pub fn init() {
    debug!("particle_xenon::init()");
    nrf52::init();
}
