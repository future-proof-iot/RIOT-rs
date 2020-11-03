#![no_main]
#![no_std]

use riot_rs_rt::debug::println;

use riot_rs_rt as _;

#[no_mangle]
fn user_main() {
    println!("hello from user_main()").unwrap();
}
