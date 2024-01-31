#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use riot_rs::rt::debug::{exit, println, EXIT_SUCCESS};

#[riot_rs::thread]
fn main() {
    println!(
        "riot_rs::thread::lock::Lock: {}",
        core::mem::size_of::<riot_rs::thread::lock::Lock>(),
    );
    println!(
        "riot_rs::thread::Thread: {}",
        core::mem::size_of::<riot_rs::thread::Thread>(),
    );

    exit(EXIT_SUCCESS);
}
