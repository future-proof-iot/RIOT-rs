#![no_main]
#![no_std]

use riot_rs as _;
use riot_rs::rt::debug::{exit, println};

#[no_mangle]
fn riot_main() {
    println!(
        "riot_rs::core::lock::Lock: {}",
        core::mem::size_of::<riot_rs::core::lock::Lock>()
    );
    println!(
        "riot_rs::core::thread::Thread: {}",
        core::mem::size_of::<riot_rs::core::thread::Thread>()
    );

    exit(0);
}
