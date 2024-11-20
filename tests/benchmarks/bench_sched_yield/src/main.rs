#![no_main]
#![no_std]
#![feature(used_with_arg)]

use ariel_os::{debug::println, thread};

#[ariel_os::thread(autostart)]
fn thread0() {
    match ariel_os::bench::benchmark(10000, || thread::yield_same()) {
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

#[ariel_os::thread(autostart)]
fn thread1() {
    loop {
        thread::yield_same()
    }
}
