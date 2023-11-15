#![no_main]
#![no_std]

use riot_rs as _;
use riot_rs::rt::debug::{exit, println, EXIT_SUCCESS};

#[no_mangle]
fn riot_main() {
    println!(
        "riot_rs::thread::lock::Lock: {}",
        core::mem::size_of::<riot_rs::thread::lock::Lock>()
    );
    println!(
        "riot_rs::thread::Thread: {}",
        core::mem::size_of::<riot_rs::thread::Thread>()
    );

    exit(EXIT_SUCCESS);
}
