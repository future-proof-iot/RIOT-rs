#![no_main]
#![no_std]
#![feature(used_with_arg)]

use ariel_os::debug::log::*;

#[ariel_os::thread(autostart)]
fn thread0() {
    info!("Hello from thread 0");
}

// `stacksize` and `priority` can be arbitrary expressions.
#[ariel_os::thread(autostart, stacksize = 4096, priority = 2)]
fn thread1() {
    info!("Hello from thread 1");
}
