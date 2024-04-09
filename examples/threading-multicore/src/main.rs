#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use riot_rs::debug::println;

#[riot_rs::thread(autostart)]
fn thread0() {
    let cpu = riot_rs::thread::cpuid();
    // let thread_id = riot_rs::thread::current_pid().unwrap();
    println!("Hello from thread {} on CPU {}", 0, cpu);
    loop {}
}

#[riot_rs::thread(autostart)]
fn thread1() {
    let cpu = riot_rs::thread::cpuid();
    // let thread_id = riot_rs::thread::current_pid().unwrap();
    println!("Hello from thread {} on CPU {}", 1, cpu);
    loop {}
}
