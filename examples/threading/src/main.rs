#![no_main]
#![no_std]
#![feature(used_with_arg)]

use riot_rs::debug::log::*;

#[riot_rs::thread(autostart)]
fn thread0() {
    info!("Hello from thread 0");
}

// `stacksize` and `priority` can be arbitrary expressions.
#[riot_rs::thread(autostart, stacksize = 4096, priority = 2)]
fn thread1() {
    info!("Hello from thread 1");
}
