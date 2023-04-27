#![no_main]
#![no_std]

use riot_rs as _;

#[cfg(not(feature = "riot-wrappers"))]
use riot_rs::rt::debug::println;

#[cfg(feature = "riot-wrappers")]
use riot_wrappers::println;

use riot_rs::rt::debug::exit;

#[no_mangle]
fn riot_main() {
    println!(
        "Hello from riot_main()! Running on a {} board.",
        riot_rs::buildinfo::BOARD
    );
    exit(0);
}
