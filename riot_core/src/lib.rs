#![no_std]
#![cfg_attr(test, no_main)]
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

use cortex_m;
use cortex_m_rt::{entry, exception, ExceptionFrame};

//use cortex_m_semihosting::hio::hstdout::;
pub use testing;

pub mod console;
pub mod runqueue;
pub mod thread;

#[exception]
unsafe fn HardFault(ef: &ExceptionFrame) -> ! {
    llvm_asm!("bkpt");
    // prints the exception frame as a panic message
    panic!("{:#?}", ef);
}

static mut IDLE_STACK: [u8; 256] = [0; 256];
static mut MAIN_STACK: [u8; 2048] = [0; 2048];

#[cfg(not(test))]
extern "C" {
    fn user_main();
}

#[cfg(test)]
unsafe fn user_main() {
    test_main();
}

fn idle(_arg: usize) {
    loop {
        cortex_m::asm::wfi();
    }
}

fn main_trampoline(_arg: usize) {
    unsafe {
        user_main();
    }
}

#[entry]
fn main() -> ! {
    boards::init();

    unsafe {
        thread::Thread::create(&mut IDLE_STACK, idle, 0, 0);
        thread::Thread::create(&mut MAIN_STACK, main_trampoline, 1, 5).jump_to();
    }

    loop {}
}
