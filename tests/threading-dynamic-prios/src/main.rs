#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use portable_atomic::{AtomicUsize, Ordering};

use riot_rs::thread::{RunqueueId, ThreadId};

static RUN_ORDER: AtomicUsize = AtomicUsize::new(0);

static TEMP_THREAD1_PRIO: RunqueueId = RunqueueId::new(5);

#[riot_rs::thread(autostart, priority = 2)]
fn thread0() {
    let pid = riot_rs::thread::current_pid().unwrap();
    assert_eq!(riot_rs::thread::get_priority(pid), Some(RunqueueId::new(2)));

    assert_eq!(RUN_ORDER.fetch_add(1, Ordering::AcqRel), 0);

    let thread1_pid = ThreadId::new(1);
    assert_eq!(
        riot_rs::thread::get_priority(thread1_pid),
        Some(RunqueueId::new(1))
    );
    riot_rs::thread::set_priority(thread1_pid, TEMP_THREAD1_PRIO);

    // thread1 runs now.

    assert_eq!(RUN_ORDER.fetch_add(1, Ordering::AcqRel), 2);
    riot_rs::debug::log::info!("Test passed!");
    loop {}
}

#[riot_rs::thread(autostart, priority = 1)]
fn thread1() {
    // Thread can only run after thread0 increased its prio.
    assert_eq!(RUN_ORDER.fetch_add(1, Ordering::AcqRel), 1);
    // Prio is the temp increased prio.
    let pid = riot_rs::thread::current_pid().unwrap();
    assert_eq!(riot_rs::thread::get_priority(pid), Some(TEMP_THREAD1_PRIO));
    // Other thread prios didn't change.
    assert_eq!(
        riot_rs::thread::get_priority(ThreadId::new(0)),
        Some(RunqueueId::new(2))
    );

    // Reset priority.
    riot_rs::thread::set_priority(pid, RunqueueId::new(1));

    unreachable!("Core should be blocked by higher prio thread.")
}
