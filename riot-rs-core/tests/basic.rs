#![no_main]
#![no_std]
// testing
#![feature(custom_test_frameworks)]
#![test_runner(testing::test_runner)]
#![reexport_test_harness_main = "test_main"]

use riot_rs_rt as _;
use riot_rs_sched::thread::Thread;

#[no_mangle]
extern "C" fn user_main() {
    #[cfg(test)]
    test_main();
}

#[test_case]
fn test_trivial() {
    assert!(1 == 1);
}

#[test_case]
fn test_pid_is_one() {
    assert!(Thread::current().pid == 1);
}
