#![cfg(not(tests))]
//
#![no_std]
//
#![allow(incomplete_features)]
// - const_generics

// features
#![feature(llvm_asm)]
#![feature(naked_functions)]
#![feature(const_generics)]
#![feature(fn_traits)]
#![feature(in_band_lifetimes)]
// clist / memoffset
#![feature(raw_ref_macros)]
#![feature(const_ptr_offset_from)]
#![feature(const_raw_ptr_deref)]
#![feature(const_maybe_uninit_as_ptr)]
// testing
#![feature(custom_test_frameworks)]
#![test_runner(testing::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate cortex_m;
extern crate cortex_m_rt as rt;

use rt::entry;
use rt::exception;
///
// dev profile: easier to debug panics; can put a breakpoint on `rust_begin_unwind`
// #[cfg(debug_assertions)]
// use panic_halt as _;

// release profile: minimize the binary size of the application
// #[cfg(not(debug_assertions))]
// use panic_abort as _;

// makes `panic!` print messages to the host stderr using semihosting
//#[cfg(not(test))]
//extern crate panic_semihosting;

//extern crate cortex_m_semihosting;

//use cortex_m_semihosting::hio::hstdout::;
pub use testing;

pub mod runqueue;
pub mod thread;

#[exception]
fn HardFault(ef: &cortex_m_rt::ExceptionFrame) -> ! {
    unsafe {
        llvm_asm!("bkpt");
    }
    // prints the exception frame as a panic message
    panic!("{:#?}", ef);
}

static mut IDLE_STACK: [u8; 256] = [0; 256];
static mut MAIN_STACK: [u8; 2048] = [0; 2048];

#[cfg(not(test))]
extern "C" {
    fn user_main();
}

#[allow(unused_variables)]
fn idle(arg: usize) {
    loop {
        cortex_m::asm::wfi();
    }
}

#[allow(unused_variables)]
fn main_trampoline(arg: usize) {
    //   #[cfg(test)]
    // test_main();

    //#[cfg(not(test))]
    unsafe {
        user_main();
    }
}

#[entry]
fn main() -> ! {
    unsafe {
        thread::Thread::create(&mut IDLE_STACK, idle, 0, 0);
        thread::Thread::create(&mut MAIN_STACK, main_trampoline, 1, 5).jump_to();
    }

    loop {}
}
