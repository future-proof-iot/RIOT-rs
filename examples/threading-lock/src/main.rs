#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use embassy_time::{Duration, Timer};
use riot_rs::{debug::log::*, thread::lock::Lock};

static LOCK: Lock = Lock::new();

#[riot_rs::task(autostart)]
async fn task_with_timer() {
    info!("Task waits for lock");
    LOCK.acquire();
    info!("Task got lock");
    info!("Task waiting for timer now...");
    Timer::after(Duration::from_secs(3)).await;
    LOCK.release();
    info!("Task released lock");
}

#[riot_rs::thread(autostart)]
fn thread0() {
    let pid = riot_rs::thread::current_pid().unwrap();
    info!("{} waits for lock", pid);
    LOCK.acquire();
    info!("{} got lock", pid);
    LOCK.release();
    info!("{} released lock", pid);
}

#[riot_rs::thread(autostart)]
fn thread1() {
    let pid = riot_rs::thread::current_pid().unwrap();
    info!("{} waits for lock", pid);
    LOCK.acquire();
    info!("{} got lock", pid);
    LOCK.release();
    info!("{} released lock", pid);
}
