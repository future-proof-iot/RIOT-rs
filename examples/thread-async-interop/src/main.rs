#![no_main]
#![no_std]
#![feature(impl_trait_in_assoc_type)]
#![feature(used_with_arg)]

use riot_rs::debug::{exit, log::*};

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use riot_rs::{blocker, EXECUTOR};

static SIGNAL: Signal<CriticalSectionRawMutex, u32> = Signal::new();

// This is a regular task.
// For this example, we don't autostart it, but let the thread spawn it.
#[riot_rs::task()]
async fn async_task() {
    use embassy_time::{Duration, Timer, TICK_HZ};
    let mut counter = 0u32;
    loop {
        if counter % 2 == 0 {
            info!("async_task() signalling");
            SIGNAL.signal(counter);
        } else {
            info!("async_task()");
        }
        Timer::after(Duration::from_ticks(TICK_HZ / 10)).await;
        counter += 1;
    }
}

#[riot_rs::thread(autostart)]
fn main() {
    use embassy_time::Instant;

    info!(
        "Hello from main()! Running on a {} board.",
        riot_rs::buildinfo::BOARD,
    );

    // Here we spawn our task.
    let spawner = EXECUTOR.spawner();
    spawner.spawn(async_task()).unwrap();

    for _ in 0..10 {
        // With `block_on()`, async functions can be called from a thread.
        // This way, async primitives like `Signal` can be used.
        let val = blocker::block_on(SIGNAL.wait());
        info!(
            "now={}ms threadtest() val={}",
            Instant::now().as_millis(),
            val,
        );
    }

    exit(Ok(()));
}
