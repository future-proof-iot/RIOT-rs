#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use portable_atomic::{AtomicUsize, Ordering};
use riot_rs::thread::{lock::Lock, thread_flags, ThreadId};

static LOCK: Lock = Lock::new();
static RUN_ORDER: AtomicUsize = AtomicUsize::new(0);
static LOCK_ORDER: AtomicUsize = AtomicUsize::new(0);

#[riot_rs::thread(autostart, priority = 1)]
fn thread0() {
    assert_eq!(RUN_ORDER.fetch_add(1, Ordering::AcqRel), 0);

    LOCK.acquire();

    // Unblock other threads in the order of their IDs.
    //
    // Because all other threads have higher priorities, setting
    // a flag will each time cause a context switch and give each
    // thread the chance to run and try acquire the lock.
    thread_flags::set(ThreadId::new(1), 0b1);
    thread_flags::set(ThreadId::new(2), 0b1);
    thread_flags::set(ThreadId::new(3), 0b1);

    assert_eq!(LOCK_ORDER.fetch_add(1, Ordering::AcqRel), 0);

    LOCK.release();

    // Wait for other threads to complete.
    thread_flags::wait_all(0b111);
    riot_rs::debug::log::info!("Test passed!");
}

#[riot_rs::thread(autostart, priority = 2)]
fn thread1() {
    thread_flags::wait_one(0b1);
    assert_eq!(RUN_ORDER.fetch_add(1, Ordering::AcqRel), 1);

    LOCK.acquire();
    assert_eq!(LOCK_ORDER.fetch_add(1, Ordering::AcqRel), 2);
    LOCK.release();

    thread_flags::set(ThreadId::new(0), 0b1);
}

#[riot_rs::thread(autostart, priority = 3)]
fn thread2() {
    thread_flags::wait_one(0b1);
    assert_eq!(RUN_ORDER.fetch_add(1, Ordering::AcqRel), 2);

    LOCK.acquire();
    // Expect to be the second thread that acquires the lock.
    assert_eq!(LOCK_ORDER.fetch_add(1, Ordering::AcqRel), 1);
    LOCK.release();

    thread_flags::set(ThreadId::new(0), 0b10);
}

#[riot_rs::thread(autostart, priority = 2)]
fn thread3() {
    thread_flags::wait_one(0b1);
    assert_eq!(RUN_ORDER.fetch_add(1, Ordering::AcqRel), 3);

    LOCK.acquire();
    assert_eq!(LOCK_ORDER.fetch_add(1, Ordering::AcqRel), 3);
    LOCK.release();

    thread_flags::set(ThreadId::new(0), 0b100);
}
