#![cfg_attr(not(test), no_std)]
#![cfg_attr(test, no_main)]
//
#![allow(incomplete_features)]
// - const_generics

// features
// linkme
#![feature(used_with_arg)]

#[cfg(feature = "threading")]
mod threading;

#[cfg(all(feature = "single-core", feature = "multi-core"))]
compile_error!(
    "feature \"single-core\" and feature \"multi-core\" cannot be enabled at the same time"
);

use ariel_os_debug::log::debug;

cfg_if::cfg_if! {
    if #[cfg(context = "cortex-m")] {
        mod cortexm;
        use cortexm as arch;
    }
    else if #[cfg(context = "esp")] {
        mod esp;
        use esp as arch;
    }
    else if #[cfg(context = "ariel-os")] {
        // When run with laze but the MCU family is not supported
        compile_error!("no runtime is defined for this MCU family");
    } else {
        // Provide a default implementation, for arch-independent tooling
        mod arch {
            #[cfg_attr(not(context = "ariel-os"), allow(dead_code))]
            pub fn init() {}
        }
    }
}

const ISR_STACKSIZE: usize =
    ariel_os_utils::usize_from_env_or!("CONFIG_ISR_STACKSIZE", 8192, "ISR stack size (in bytes)");

#[link_section = ".isr_stack"]
#[used(linker)]
static ISR_STACK: [u8; ISR_STACKSIZE] = [0u8; ISR_STACKSIZE];

#[cfg(feature = "_panic-handler")]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    #[cfg(not(feature = "silent-panic"))]
    {
        ariel_os_debug::println!("panic: {}\n", _info);
        ariel_os_debug::exit(ariel_os_debug::ExitCode::FAILURE);
    }
    #[allow(clippy::empty_loop)]
    loop {}
}

use linkme::distributed_slice;

#[distributed_slice]
pub static INIT_FUNCS: [fn()] = [..];

#[inline]
#[cfg_attr(not(context = "ariel-os"), allow(dead_code))]
fn startup() -> ! {
    arch::init();

    #[cfg(feature = "debug-console")]
    ariel_os_debug::init();

    debug!("ariel_os_rt::startup()");

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
            fn __ariel_os_embassy_init() -> !;
        }
        debug!("ariel_os_rt::startup() launching single thread executor");
        unsafe { __ariel_os_embassy_init() };
    }

    #[cfg(not(any(feature = "threading", feature = "executor-single-thread")))]
    {
        #[cfg(test)]
        test_main();
        #[allow(clippy::empty_loop)]
        loop {}
    }
}
