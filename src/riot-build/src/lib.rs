#![no_std]

use riot_rs_rt::debug::println;

#[cfg(feature = "riot-rs-core")]
use riot_rs_core as _;

#[no_mangle]
fn riot_rs_rt_startup() {
    extern "C" {
        pub fn board_init();
        pub fn kernel_init();
    }

    println!("riot_build::riot_startup(): launching RIOT startup");
    unsafe {
        board_init();
        //libc_init();
        kernel_init();
    }
}
