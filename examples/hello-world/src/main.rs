#![no_main]
#![no_std]
#![feature(impl_trait_in_assoc_type)]
#![feature(used_with_arg)]

use ariel_os::debug::{exit, log::*};

#[ariel_os::task(autostart)]
async fn main() {
    info!("Hello World!");

    exit(Ok(()));
}
