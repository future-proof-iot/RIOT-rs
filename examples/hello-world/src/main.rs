#![no_main]
#![no_std]

use riot_rs as _;
use riot_rs::rt::debug::println;

#[no_mangle]
fn riot_main() {
    println!("hello from riot_main()!");
}
