#![no_main]
#![no_std]
#![feature(used_with_arg)]

use ariel_os::debug::{exit, log::*, EXIT_SUCCESS};

#[ariel_os::thread(autostart)]
fn main() {
    info!(
        "ariel_os::thread::sync::Lock: {}",
        core::mem::size_of::<ariel_os::thread::sync::Lock>(),
    );
    info!(
        "ariel_os::thread::Thread: {}",
        ariel_os::thread::thread_struct_size()
    );

    exit(EXIT_SUCCESS);
}
