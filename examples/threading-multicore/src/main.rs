#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use riot_rs::debug::println;

#[riot_rs::thread(autostart)]
fn thread0() {
    let cpu = riot_rs::thread::cpuid();
    let thread_id = riot_rs::thread::current_pid().unwrap();
    println!(
        "[Thread {} on Core {}] Hello from a low-priority thread!",
        thread_id, cpu
    );
}

#[riot_rs::thread(autostart, priority = 2)]
fn thread1() {
    let cpu = riot_rs::thread::cpuid();
    let thread_id = riot_rs::thread::current_pid().unwrap();
    println!(
        "[Thread {} on Core {}] Hello from a medium-priority thread! I am looping forever now...",
        thread_id, cpu
    );
    loop {}
}

#[riot_rs::thread(autostart, priority = 3)]
fn thread2() {
    let cpu = riot_rs::thread::cpuid();
    let thread_id = riot_rs::thread::current_pid().unwrap();
    println!(
        "[Thread {} on Core {}] Hello from a high-priority thread!",
        thread_id, cpu
    );
}
