#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use ariel_os::{
    debug::log::*,
    thread::{current_pid, sync::Channel, thread_flags, ThreadId},
};

static ID_EXCHANGE: Channel<ThreadId> = Channel::new();

#[ariel_os::thread(autostart)]
fn thread0() {
    let target_pid = ID_EXCHANGE.recv();
    ID_EXCHANGE.send(&current_pid().unwrap());

    match ariel_os::bench::benchmark(1000, || {
        thread_flags::set(target_pid, 1);
        thread_flags::wait_any(1);
    }) {
        Ok(ticks) => info!("took {} ticks per iteration", ticks),
        Err(_) => warn!("benchmark returned error"),
    }
}

#[ariel_os::thread(autostart)]
fn thread1() {
    ID_EXCHANGE.send(&current_pid().unwrap());
    let target_pid = ID_EXCHANGE.recv();

    loop {
        thread_flags::set(target_pid, 1);
        thread_flags::wait_any(1);
    }
}
