#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use riot_rs::{debug::println, thread};

#[riot_rs::thread(autostart)]
fn thread0() {
    match riot_rs::rt::benchmark(10000, || thread::yield_same()) {
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
        thread::yield_same()
    }
}
