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
// ringbuffer
#![feature(const_fn)]
#![feature(const_mut_refs)]
// for msg_content_t union
// error[E0658]: unions with non-`Copy` fields other than `ManuallyDrop<T>` are unstable
#![feature(untagged_unions)]
// for THREAD_MSG_WAITERS static initialization
#![feature(const_in_array_repeat_expressions)]
// testing
#![feature(custom_test_frameworks)]
#![test_runner(riot_rs_rt::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]

pub mod channel;
pub mod event_group;
pub mod lock;
pub mod mutex;
pub mod runqueue;
pub mod thread;

#[cfg(test)]
pub mod startup;

#[cfg(test)]
mod test {
    use riot_rs_rt as _;

    pub fn startup() {
        crate::startup::startup();
    }
}

#[cfg(test)]
#[no_mangle]
extern "C" fn user_main() {
    test_main();
}

#[cfg(test)]
#[no_mangle]
extern "C" fn riot_rs_rt_startup() {
    riot_rs_boards::init();
    test::startup();
}

#[test_case]
fn test_trivial() {
    assert!(1 == 1);
}
