#![no_main]
#![no_std]
#![feature(used_with_arg)]

use ariel_os::debug::{exit, log::*, ExitCode};

#[ariel_os::thread(autostart)]
fn main() {
    info!("Available information:");
    info!("Board type: {}", ariel_os::buildinfo::BOARD);
    if let Ok(id) = ariel_os::identity::device_id_bytes() {
        info!("Device ID: {=[u8]:02x}", id.as_ref());
    } else {
        info!("Device ID is unavailable.");
    }
    if let Ok(eui48) = ariel_os::identity::interface_eui48(0) {
        info!("Device's first EUI-48 address: {}", eui48);
    }

    exit(ExitCode::SUCCESS);
}
