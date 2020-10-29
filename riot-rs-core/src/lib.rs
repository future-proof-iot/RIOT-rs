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
// for msg_content_t union
// error[E0658]: unions with non-`Copy` fields other than `ManuallyDrop<T>` are unstable
#![feature(untagged_unions)]
// testing
#![feature(custom_test_frameworks)]
#![test_runner(testing::test_runner)]
#![reexport_test_harness_main = "test_main"]

pub mod runqueue;
pub mod thread;

#[cfg(test)]
use riot_rs_rt as _;

#[no_mangle]
#[cfg(test)]
extern "C" fn user_main() {
    #[cfg(test)]
    test_main();
}

#[test_case]
fn test_trivial() {
    assert!(1 == 1);
}
