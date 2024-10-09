#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use riot_rs::{
    debug::log::*,
    thread::{sync::Channel, yield_same, CoreAffinity, CoreId},
};

static CHANNEL: Channel<bool> = Channel::new();

#[riot_rs::thread(autostart, priority = 1)]
fn thread0() {
    let core = riot_rs::thread::core_id();
    let pid = riot_rs::thread::current_pid().unwrap();
    info!("Hello from low prio thread {:?} on core {:?}", pid, core);
    loop {}
}

#[riot_rs::thread(autostart, priority = 2)]
fn thread1() {
    let core = riot_rs::thread::core_id();
    let pid = riot_rs::thread::current_pid().unwrap();
    info!("Hello from mid prio thread {:?} on core {:?}", pid, core);
    yield_same();
    info!("{:?} running again on core {:?}", pid, core);
}

#[riot_rs::thread(autostart, priority = 2)]
fn thread2() {
    let core = riot_rs::thread::core_id();
    let pid = riot_rs::thread::current_pid().unwrap();
    info!("Hello from mid prio thread {:?} on core {:?}", pid, core);
    yield_same();
    CHANNEL.send(&true);
    info!("{:?} running again (forever) on core {:?}", pid, core);
    loop {}
}

#[riot_rs::thread(autostart, priority = 3, affinity = CoreAffinity::one(CoreId::new(1)))]
fn thread3() {
    let core = riot_rs::thread::core_id();
    let pid = riot_rs::thread::current_pid().unwrap();
    info!("Hello from high prio thread {:?} on core {:?}", pid, core);
}

#[riot_rs::thread(autostart, priority = 3, affinity = CoreAffinity::one(CoreId::new(1)))]
fn thread4() {
    let core = riot_rs::thread::core_id();
    let pid = riot_rs::thread::current_pid().unwrap();
    info!("Hello from high prio thread {:?} on core {:?}", pid, core);
    while CHANNEL.try_recv().is_none() {}
}
