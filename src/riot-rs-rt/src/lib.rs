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

#[cfg(feature = "threading")]
mod threading;

use riot_rs_debug::println;

cfg_if::cfg_if! {
    if #[cfg(context = "cortex-m")] {
        mod cortexm;
        use cortexm as arch;
    }
    else if #[cfg(context = "esp")] {
        mod esp;
        use esp as arch;
    }
    else if #[cfg(context = "riot-rs")] {
        // When run with laze but the architecture is not supported
        compile_error!("no runtime is defined for this architecture");
    } else {
        // Provide a default architecture, for arch-independent tooling
        mod arch {
            #[cfg_attr(not(context = "riot-rs"), allow(dead_code))]
            pub fn init() {}
        }
    }
}

const ISR_STACKSIZE: usize =
    riot_rs_utils::usize_from_env_or!("CONFIG_ISR_STACKSIZE", 8192, "ISR stack size (in bytes)");

#[link_section = ".isr_stack"]
#[used(linker)]
static ISR_STACK: [u8; ISR_STACKSIZE] = [0u8; ISR_STACKSIZE];

#[cfg(feature = "_panic-handler")]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    #[cfg(feature = "silent-panic")]
    let _ = info;

    #[cfg(not(feature = "silent-panic"))]
    {
        println!("panic: {}\n", info);
        riot_rs_debug::exit(riot_rs_debug::EXIT_FAILURE);
    }
    #[allow(clippy::empty_loop)]
    loop {}
}

use linkme::distributed_slice;

#[distributed_slice]
pub static INIT_FUNCS: [fn()] = [..];

#[inline]
#[cfg_attr(not(context = "riot-rs"), allow(dead_code))]
fn startup() -> ! {
    arch::init();

    #[cfg(feature = "debug-console")]
    riot_rs_debug::init();

    println!("riot_rs_rt::startup()");

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

    #[cfg(feature = "executor-single-thread")]
    {
        extern "Rust" {
            fn riot_rs_embassy_init() -> !;
        }
        unsafe { riot_rs_embassy_init() };
    }

    #[cfg(not(any(feature = "threading", feature = "executor-single-thread")))]
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
