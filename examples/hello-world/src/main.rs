#![no_main]
#![no_std]

use riot_rs_rt::debug::println;

use riot_build as _;
use riot_rs_rt as _;

#[no_mangle]
fn riot_main() {
    println!("hello from riot_main()!");
}
