#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use riot_rs::rt::debug::println;

#[riot_rs::thread]
fn thread0() {
    println!("Hello from thread 0");
}

#[riot_rs::thread(stacksize = 4096, priority = 2)]
fn thread1() {
    println!("Hello from thread 1");
}

#[no_mangle]
fn riot_main() {
    println!(
        "Hello from riot_main()! Running on a {} board.",
        riot_rs::buildinfo::BOARD
    );
}
