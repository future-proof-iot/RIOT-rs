#![no_main]
#![no_std]
#![feature(used_with_arg)]

use riot_rs::debug::{exit, log::*};

#[riot_rs::thread(autostart)]
fn main() {
    info!("Available information:");
    info!("Board type: {}", riot_rs::buildinfo::BOARD);
    if let Ok(id) = riot_rs::identity::device_id_bytes() {
        info!("Device ID: {=[u8]:02x}", id.as_ref());
    } else {
        info!("Device ID is unavailable.");
    }

    exit(Ok(()));
}
