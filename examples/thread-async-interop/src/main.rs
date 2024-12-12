#![no_main]
#![no_std]
#![feature(impl_trait_in_assoc_type)]
#![feature(used_with_arg)]

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};

use ariel_os::{
    asynch::{blocker, spawner},
    debug::{exit, log::*, ExitCode},
    time::{Duration, Instant, Timer},
};

static SIGNAL: Signal<CriticalSectionRawMutex, u32> = Signal::new();

// This is a regular task.
// For this example, we don't autostart it, but let the thread spawn it.
#[ariel_os::task()]
async fn async_task() {
    info!("async_task(): starting");

    let mut counter = 0u32;
    loop {
        info!("async_task(): signalling, counter={}", counter);
        SIGNAL.signal(counter);
        Timer::after(Duration::from_millis(100)).await;
        counter += 1;
    }
}

#[ariel_os::thread(autostart)]
fn main() {
    info!("main(): starting");

    // Here we spawn our task.
    spawner().spawn(async_task()).unwrap();

    for _ in 0..10 {
        // With `block_on()`, async functions can be called from a thread.
        // This way, async primitives like `Signal` can be used.
        let counter = blocker::block_on(SIGNAL.wait());

        // Get time since boot
        let now = Instant::now().as_millis();
        info!("main(): now={}ms threadtest() counter={}", now, counter);
    }

    info!("main(): all good, exiting.");

    exit(ExitCode::SUCCESS);
}
