#![no_main]
#![no_std]
// testing
#![feature(custom_test_frameworks)]
#![test_runner(riot_core::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]

use riot_core::thread::Thread;

#[no_mangle]
extern "C" fn user_main() {
    #[cfg(test)]
    test_main();
}

#[test_case]
fn test_foo() {
    assert!(1 == 1);
}

#[test_case]
fn test_pid_zero() {
    assert!(Thread::current().pid == 1);
}
