#![no_main]
#![no_std]
#![feature(impl_trait_in_assoc_type)]
#![feature(used_with_arg)]

use riot_rs::debug::{exit, log::*};

#[riot_rs::task(autostart)]
async fn main() {
    info!("Hello World!");

    exit(Ok(()));
}
