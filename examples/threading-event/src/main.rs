#![no_main]
#![no_std]
#![feature(used_with_arg)]

use ariel_os::debug::{log::*, ExitCode};
use ariel_os::thread::{sync::Event, ThreadId};

static EVENT: Event = Event::new();

fn waiter() {
    let my_id = ariel_os::thread::current_pid().unwrap();
    let my_prio = ariel_os::thread::get_priority(my_id).unwrap();
    info!("[{:?}@{}] Waiting for event...", my_id, my_prio);
    EVENT.wait();
    info!("[{:?} Done.", my_id);

    if my_id == ThreadId::new(0) {
        info!("All five threads should have reported \"Done.\". exiting.");
        ariel_os::debug::exit(ExitCode::SUCCESS);
    }
}

#[ariel_os::thread(autostart, priority = 0)]
fn thread0() {
    waiter();
}

#[ariel_os::thread(autostart, priority = 1)]
fn thread1() {
    waiter();
}

#[ariel_os::thread(autostart, priority = 2)]
fn thread2() {
    waiter();
}

#[ariel_os::thread(autostart, priority = 3)]
fn thread3() {
    waiter();
}

#[ariel_os::thread(autostart, priority = 1)]
fn thread4() {
    let my_id = ariel_os::thread::current_pid().unwrap();
    let my_prio = ariel_os::thread::get_priority(my_id).unwrap();
    info!("[{:?}@{}] Setting event...", my_id, my_prio);
    EVENT.set();
    info!("[{:?}@{}] Event set.", my_id, my_prio);
    waiter();
}
