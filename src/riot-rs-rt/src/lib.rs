#![cfg_attr(not(test), no_std)]
#![cfg_attr(test, no_main)]
//
#![allow(incomplete_features)]
// - const_generics

// features
#![feature(naked_functions)]
#![feature(fn_traits)]
// testing
#![feature(custom_test_frameworks)]
#![test_runner(crate::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]
pub mod testing;

use core::panic::PanicInfo;

pub mod debug;
pub use debug::*;

cfg_if::cfg_if! {
    if #[cfg(all(target_arch = "arm", target_feature = "thumb2"))] {
        mod cortexm;
        use cortexm as arch;
    }
    else {
        mod arch {
            pub fn init() {}
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    #[cfg(not(feature = "silent-panic"))]
    {
        debug::println!("panic: {}\n", _info);
        debug::exit(debug::EXIT_FAILURE);
    }
    loop {}
}

#[cfg(not(test))]
extern "C" {
    fn riot_rs_rt_startup();
}

#[inline]
fn startup() -> ! {
    arch::init();

    #[cfg(feature = "debug-console")]
    debug::init();

    debug::println!("riot_rs_rt::main()");

    #[cfg(not(test))]
    unsafe {
        riot_rs_rt_startup();
    }

    #[cfg(test)]
    test_main();
    loop {}
}

#[test_case]
fn test_trivial() {
    assert!(1 == 1);
}
