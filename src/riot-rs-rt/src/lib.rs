#![cfg_attr(not(test), no_std)]
#![cfg_attr(test, no_main)]
//
#![allow(incomplete_features)]
// - const_generics

// features
#![feature(naked_functions)]
#![feature(fn_traits)]
// linkme
#![feature(used_with_arg)]
// testing
#![feature(custom_test_frameworks)]
#![test_runner(crate::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]
pub mod testing;

pub mod debug;
pub use debug::*;

#[cfg(feature = "threading")]
mod threading;

cfg_if::cfg_if! {
    if #[cfg(context = "cortex-m")] {
        mod cortexm;
        use cortexm as arch;
    }
    else {
        mod arch {
            pub fn init() {}
            pub fn benchmark<F: Fn()>(_iterations: usize, _f: F) -> core::result::Result<usize, ()> {
                unimplemented!();
            }
        }
    }
}

pub use arch::benchmark;

const ISR_STACKSIZE: usize =
    riot_rs_utils::usize_from_env_or!("CONFIG_ISR_STACKSIZE", 8192, "ISR stack size (in bytes)");

#[link_section = ".isr_stack"]
#[used(linker)]
static ISR_STACK: [u8; ISR_STACKSIZE] = [0u8; ISR_STACKSIZE];

#[cfg(feature = "_panic-handler")]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    #[cfg(not(feature = "silent-panic"))]
    {
        debug::println!("panic: {}\n", _info);
        debug::exit(debug::EXIT_FAILURE);
    }
    #[allow(clippy::empty_loop)]
    loop {}
}

use linkme::distributed_slice;

#[distributed_slice]
pub static INIT_FUNCS: [fn()] = [..];

#[inline]
fn startup() -> ! {
    arch::init();

    #[cfg(feature = "debug-console")]
    debug::init();

    debug::println!("riot_rs_rt::startup()");

    for f in INIT_FUNCS {
        f();
    }

    #[cfg(feature = "threading")]
    {
        // SAFETY: this function must not be called more than once
        unsafe {
            threading::start();
        }
    }

    #[cfg(not(feature = "threading"))]
    {
        #[cfg(test)]
        test_main();
        #[allow(clippy::empty_loop)]
        loop {}
    }
}

#[test_case]
fn test_trivial() {
    assert!(1 == 1);
}
