#![no_main]
#![no_std]
#![feature(used_with_arg)]

use riot_rs::debug::{exit, log::*};

#[riot_rs::thread(autostart)]
fn main() {
    info!("Hello World!");

    exit(Ok(()));
}
