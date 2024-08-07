#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use riot_rs::{debug::log::*, thread::ThreadId};

#[riot_rs::thread(autostart, priority = 3)]
fn thread0() {
    let pid = riot_rs::thread::current_pid().unwrap();
    info!(
        "{}: Running at prio {}.",
        pid,
        riot_rs::thread::get_priority(pid).unwrap()
    );
    let new_thread1_prio = 5;
    info!("{}: Changing Thread 1's prio to {}.", pid, new_thread1_prio);
    riot_rs::thread::set_priority(ThreadId::new(1), new_thread1_prio);
    info!(
        "{}: Running again at prio {}.",
        pid,
        riot_rs::thread::get_priority(pid).unwrap()
    );
}

#[riot_rs::thread(autostart, priority = 1)]
fn thread1() {
    let pid = riot_rs::thread::current_pid().unwrap();
    info!(
        "{}: Running at prio {}.",
        pid,
        riot_rs::thread::get_priority(pid).unwrap()
    );
    let new_prio = 1;
    info!("{}: Changing own prio back to {}.", pid, new_prio);
    riot_rs::thread::set_priority(pid, new_prio);
    info!(
        "{}: Running again at prio {}.",
        pid,
        riot_rs::thread::get_priority(pid).unwrap()
    );
}
