#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use riot_rs::debug::{exit, println};

#[riot_rs::thread(autostart)]
fn main() {
    println!(
        "Hello from main()! Running on a {} board.",
        riot_rs::buildinfo::BOARD,
    );

    exit(Ok(()));
}
