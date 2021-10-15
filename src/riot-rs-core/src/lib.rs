//! RIOT-rs kernel implementation
//!
//! This module implements the RIOT-rs kernel functionality. It contains the
//! implementations of the scheduler and task switching, thread handling, messaging,
//! thread flags and locking.
//!
//! It currently also contains the implementation of the C bindings / glue code
//! for the corresponding RIOT modules thread, msg, mutex, and thread_flags.

#![no_std]
#![cfg_attr(test, no_main)]
//
#![allow(incomplete_features)]
// - const_generics

// features
#![feature(asm)]
#![feature(naked_functions)]
#![feature(fn_traits)]
#![feature(in_band_lifetimes)]
#![feature(inline_const)] // for THREAD_MSG_CHANNELS initialization
#![feature(option_result_unwrap_unchecked)]
#![feature(const_fn_trait_bound)]
// clist / memoffset
#![feature(const_ptr_offset_from)]
#![feature(const_raw_ptr_deref)]
#![feature(const_maybe_uninit_as_ptr)]
// ringbuffer
#![feature(const_mut_refs)]
// for msg_content_t union
// error[E0658]: unions with non-`Copy` fields other than `ManuallyDrop<T>` are unstable
#![feature(untagged_unions)]
// testing
#![feature(custom_test_frameworks)]
#![test_runner(riot_rs_rt::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]

pub mod channel;
pub mod event_group;
pub mod lock;
pub mod mutex;
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
