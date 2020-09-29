// test prelude
#![no_main]
#![no_std]
#![feature(custom_test_frameworks)]
#![test_runner(riot_core::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[no_mangle]
extern "C" fn user_main() {
    #[cfg(test)]
    test_main();
}

// test prelude end

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

#[test_case]
fn test_rq_basic_twoprio() {
    let mut runqueue: RunQueue<8> = RunQueue::new();

    runqueue.add(0, 0);
    runqueue.add(1, 0);
    runqueue.add(3, 0);

    runqueue.add(2, 1);
    runqueue.add(4, 1);

    assert!(runqueue.get_next() == 4);
    runqueue.del(4, 1);
    assert!(runqueue.get_next() == 2);
    runqueue.del(2, 1);
    assert!(runqueue.get_next() == 3);
    runqueue.del(3, 0);
    assert!(runqueue.get_next() == 1);
    runqueue.del(1, 0);
    assert!(runqueue.get_next() == 0);
    runqueue.del(0, 0);
}
