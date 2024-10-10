#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use portable_atomic::{AtomicUsize, Ordering};
use riot_rs::thread::{sync::Mutex, thread_flags, RunqueueId, ThreadId};

static MUTEX: Mutex<usize> = Mutex::new(0);
static RUN_ORDER: AtomicUsize = AtomicUsize::new(0);

#[riot_rs::thread(autostart, priority = 1)]
fn thread0() {
    let pid = riot_rs::thread::current_pid().unwrap();
    assert_eq!(riot_rs::thread::get_priority(pid), Some(RunqueueId::new(1)));

    assert_eq!(RUN_ORDER.fetch_add(1, Ordering::AcqRel), 0);

    let mut counter = MUTEX.lock();

    // Unblock other threads in the order of their IDs.
    //
    // Because all other threads have higher priorities, setting
    // a flag will each time cause a context switch and give each
    // thread the chance to run and try acquire the lock.
    thread_flags::set(ThreadId::new(1), 0b1);
    // Inherit prio of higher prio waiting thread.
    assert_eq!(
        riot_rs::thread::get_priority(pid),
        riot_rs::thread::get_priority(ThreadId::new(1)),
    );
    thread_flags::set(ThreadId::new(2), 0b1);
    // Inherit prio of highest waiting thread.
    assert_eq!(
        riot_rs::thread::get_priority(pid),
        riot_rs::thread::get_priority(ThreadId::new(2)),
    );
    thread_flags::set(ThreadId::new(3), 0b1);
    // Still has priority of highest waiting thread.
    assert_eq!(
        riot_rs::thread::get_priority(pid),
        riot_rs::thread::get_priority(ThreadId::new(2)),
    );

    assert_eq!(*counter, 0);
    *counter += 1;

    drop(counter);

    // Return to old prio.
    assert_eq!(riot_rs::thread::get_priority(pid), Some(RunqueueId::new(1)));

    // Wait for other threads to complete.
    thread_flags::wait_all(0b111);

    assert_eq!(*MUTEX.lock(), 4);
    riot_rs::debug::log::info!("Test passed!");
}

#[riot_rs::thread(autostart, priority = 2)]
fn thread1() {
    let pid = riot_rs::thread::current_pid().unwrap();
    assert_eq!(riot_rs::thread::get_priority(pid), Some(RunqueueId::new(2)));

    thread_flags::wait_one(0b1);
    assert_eq!(RUN_ORDER.fetch_add(1, Ordering::AcqRel), 1);

    let mut counter = MUTEX.lock();
    assert_eq!(*counter, 2);
    *counter += 1;

    thread_flags::set(ThreadId::new(0), 0b1);
}

#[riot_rs::thread(autostart, priority = 3)]
fn thread2() {
    let pid = riot_rs::thread::current_pid().unwrap();
    assert_eq!(riot_rs::thread::get_priority(pid), Some(RunqueueId::new(3)));

    thread_flags::wait_one(0b1);
    assert_eq!(RUN_ORDER.fetch_add(1, Ordering::AcqRel), 2);

    let mut counter = MUTEX.lock();
    assert_eq!(*counter, 1);
    // Priority didn't change because this thread has higher prio
    // than all waiting threads.
    assert_eq!(riot_rs::thread::get_priority(pid), Some(RunqueueId::new(3)),);
    *counter += 1;

    thread_flags::set(ThreadId::new(0), 0b10);
}

#[riot_rs::thread(autostart, priority = 2)]
fn thread3() {
    let pid = riot_rs::thread::current_pid().unwrap();
    assert_eq!(riot_rs::thread::get_priority(pid), Some(RunqueueId::new(2)));

    thread_flags::wait_one(0b1);
    assert_eq!(RUN_ORDER.fetch_add(1, Ordering::AcqRel), 3);

    let mut counter = MUTEX.lock();
    assert_eq!(*counter, 3);
    *counter += 1;

    thread_flags::set(ThreadId::new(0), 0b100);
}
