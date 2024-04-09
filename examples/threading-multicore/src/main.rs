#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use riot_rs::debug::log::*;

#[riot_rs::thread(autostart)]
fn thread0() {
    let core = riot_rs::thread::core_id();
    let pid = riot_rs::thread::current_pid().unwrap();
    info!("Hello from {:?} on {:?}", pid, core);
    loop {}
}

#[riot_rs::thread(autostart)]
fn thread1() {
    let core = riot_rs::thread::core_id();
    let pid = riot_rs::thread::current_pid().unwrap();
    info!("Hello from {:?} on {:?}", pid, core);
    loop {}
}
