#![no_std]

use riot_rs_rt::debug::println;

#[cfg(feature = "riot-rs-core")]
use riot_rs_core as _;

/// startup function as called by riot-rs-rt
///
/// This calls init() from riot_rs_boards, allowing board specific RIOT-rs code
/// to run.
/// It then defers to RIOT's board_init() and kernel_init().
///
/// This is the RIOT-rs equivalent of RIOT's kernel_init().
#[no_mangle]
fn riot_rs_rt_startup() {
    extern "C" {
        pub fn board_init();
        pub fn kernel_init();
    }

    /* riot-rs board initialization */
    riot_rs_boards::init();

    println!("riot_build::riot_startup(): launching RIOT startup");
    unsafe {
        board_init();
        //libc_init();
        kernel_init();
    }
}
