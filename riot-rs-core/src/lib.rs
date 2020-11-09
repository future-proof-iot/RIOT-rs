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
#![test_runner(riot_rs_rt::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]

pub mod runqueue;
pub mod thread;

#[cfg(test)]
pub mod startup;

#[cfg(test)]
mod test {
    use linkme::distributed_slice;
    use riot_rs_rt as _;
    use riot_rs_rt::init::INIT_FUNCS;

    #[distributed_slice(INIT_FUNCS, 99)]
    fn startup() {
        crate::startup::startup();
    }
}

#[cfg(test)]
#[no_mangle]
extern "C" fn user_main() {
    test_main();
}

#[test_case]
fn test_trivial() {
    assert!(1 == 1);
}
