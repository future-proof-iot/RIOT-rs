#![no_main]
#![no_std]
#![feature(used_with_arg)]

use riot_rs::debug::{exit, log::*};

#[riot_rs::thread(autostart)]
fn main() {
    info!(
        "Hello from main()! Running on {} board identified as {:x}.",
        riot_rs::buildinfo::BOARD,
        riot_rs::identity::device_identity(),
    );

    exit(Ok(()));
}
