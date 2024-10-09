#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use riot_rs::{
    debug::println,
    thread::{thread_flags, ThreadId},
};

#[riot_rs::thread(autostart)]
fn thread0() {
    match riot_rs::bench::benchmark(1000, || {
        thread_flags::set(ThreadId::new(1), 1);
        thread_flags::wait_any(1);
    }) {
        Ok(ticks) => {
            println!(
                "took {} ticks per iteration ({} per context switch)",
                ticks,
                ticks / 2
            );
        }
        Err(_) => {
            println!("benchmark returned error");
        }
    }
}

#[riot_rs::thread(autostart)]
fn thread1() {
    loop {
        thread_flags::set(ThreadId::new(0), 1);
        thread_flags::wait_any(1);
    }
}
