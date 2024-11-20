#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use portable_atomic::{AtomicUsize, Ordering};

use ariel_os::thread::{RunqueueId, ThreadId};

static RUN_ORDER: AtomicUsize = AtomicUsize::new(0);

static TEMP_THREAD1_PRIO: RunqueueId = RunqueueId::new(5);

#[ariel_os::thread(autostart, priority = 2)]
fn thread0() {
    let pid = ariel_os::thread::current_pid().unwrap();
    assert_eq!(ariel_os::thread::get_priority(pid), Some(RunqueueId::new(2)));

    assert_eq!(RUN_ORDER.fetch_add(1, Ordering::AcqRel), 0);

    let thread1_pid = ThreadId::new(1);
    assert_eq!(
        ariel_os::thread::get_priority(thread1_pid),
        Some(RunqueueId::new(1))
    );
    ariel_os::thread::set_priority(thread1_pid, TEMP_THREAD1_PRIO);

    // thread1 runs now.

    assert_eq!(RUN_ORDER.fetch_add(1, Ordering::AcqRel), 2);
    ariel_os::debug::log::info!("Test passed!");
    loop {}
}

#[ariel_os::thread(autostart, priority = 1)]
fn thread1() {
    // Thread can only run after thread0 increased its prio.
    assert_eq!(RUN_ORDER.fetch_add(1, Ordering::AcqRel), 1);
    // Prio is the temp increased prio.
    let pid = ariel_os::thread::current_pid().unwrap();
    assert_eq!(ariel_os::thread::get_priority(pid), Some(TEMP_THREAD1_PRIO));
    // Other thread prios didn't change.
    assert_eq!(
        ariel_os::thread::get_priority(ThreadId::new(0)),
        Some(RunqueueId::new(2))
    );

    // Reset priority.
    ariel_os::thread::set_priority(pid, RunqueueId::new(1));

    unreachable!("Core should be blocked by higher prio thread.")
}
