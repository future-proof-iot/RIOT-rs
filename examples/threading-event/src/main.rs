#![no_main]
#![no_std]
#![feature(used_with_arg)]

use riot_rs::debug::{log::*, EXIT_SUCCESS};
use riot_rs::thread::{sync::Event, ThreadId};

static EVENT: Event = Event::new();

fn waiter() {
    let my_id = riot_rs::thread::current_pid().unwrap();
    let my_prio = riot_rs::thread::get_priority(my_id).unwrap();
    info!("[{:?}@{}] Waiting for event...", my_id, my_prio);
    EVENT.wait();
    info!("[{:?} Done.", my_id);

    if my_id == ThreadId::new(0) {
        info!("All five threads should have reported \"Done.\". exiting.");
        riot_rs::debug::exit(EXIT_SUCCESS);
    }
}

#[riot_rs::thread(autostart, priority = 0)]
fn thread0() {
    waiter();
}

#[riot_rs::thread(autostart, priority = 1)]
fn thread1() {
    waiter();
}

#[riot_rs::thread(autostart, priority = 2)]
fn thread2() {
    waiter();
}

#[riot_rs::thread(autostart, priority = 3)]
fn thread3() {
    waiter();
}

#[riot_rs::thread(autostart, priority = 1)]
fn thread4() {
    let my_id = riot_rs::thread::current_pid().unwrap();
    let my_prio = riot_rs::thread::get_priority(my_id).unwrap();
    info!("[{:?}@{}] Setting event...", my_id, my_prio);
    EVENT.set();
    info!("[{:?}@{}] Event set.", my_id, my_prio);
    waiter();
}
