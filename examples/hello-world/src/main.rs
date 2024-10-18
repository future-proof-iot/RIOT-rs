#![no_main]
#![no_std]
#![feature(used_with_arg)]

use riot_rs::debug::{exit, log::*};

#[riot_rs::thread(autostart)]
fn main() {
    info!(
        "Hello from main()! Running on a {} board.",
        riot_rs::buildinfo::BOARD,
    );

    exit(Ok(()));
}
