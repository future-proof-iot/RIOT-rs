#![no_main]
#![no_std]
#![feature(custom_test_frameworks)]
#![test_runner(riot_rs_rt::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]

use linkme::distributed_slice;

use boards::board as _;

#[test_case]
fn test_trivial() {
    assert!(1 == 1);
}

#[distributed_slice(riot_rs_rt::init::INIT_FUNCS, 100)]
fn test() {
    test_main();
}
