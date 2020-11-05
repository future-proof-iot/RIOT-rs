#![no_std]
#![no_main]

use linkme::distributed_slice;
use riot_rs_rt::debug::println;
use riot_rs_rt::init::INIT_FUNCS;

#[cfg(feature = "riot-rs-core")]
use riot_rs_core as _;

#[distributed_slice(INIT_FUNCS, 99)]
fn riot_startup() {
    extern "C" {
        pub fn board_init();
        pub fn kernel_init();
    //pub fn libc_init();
    }

    // due to https://github.com/rust-lang/rust/issues/47384,
    // the board's additions to INIT_FUNCS get silently discarded
    // if there's not other symbol used from the modules.
    // Thus explitly link in a dummy.
    boards::linkme_please();

    println!("riot::user_main()");
    unsafe {
        board_init();
        //libc_init();
        kernel_init();
    }
}
