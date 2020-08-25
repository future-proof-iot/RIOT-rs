#![no_main]
#![no_std]
// testing
#![feature(custom_test_frameworks)]
#![test_runner(riot_core::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[no_mangle]
extern "C" fn user_main() {
    #[cfg(test)]
    test_main();
}

use riot_core::testing::println;

use riot_core::runqueue::RunQueue;

#[test_case]
fn test_rq_basic() {
    let mut runqueue: RunQueue<8> = RunQueue::new();

    runqueue.add(0, 0);
    runqueue.add(1, 0);
    runqueue.add(2, 0);

    assert!(runqueue.get_next() == 2);

    runqueue.advance(2, 0);

    assert!(runqueue.get_next() == 1);
    runqueue.advance(1, 0);

    assert!(runqueue.get_next() == 0);
    assert!(runqueue.get_next() == 0);

    runqueue.advance(1, 0);
    assert!(runqueue.get_next() == 0);

    runqueue.advance(0, 0);
    assert!(runqueue.get_next() == 2);

    runqueue.advance(2, 0);
    assert!(runqueue.get_next() == 1);
}

#[test_case]
fn test_rq_all32() {
    let mut runqueue: RunQueue<8> = RunQueue::new();

    for i in 0..=31 {
        runqueue.add(i, 0);
    }

    for i in (0..=31).rev() {
        assert!(runqueue.get_next() == i);
        runqueue.advance(i as u8, 0);
    }

    for i in (0..=31).rev() {
        assert!(runqueue.get_next() == i);
        runqueue.advance(i as u8, 0);
    }
}
