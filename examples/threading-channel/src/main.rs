#![no_main]
#![no_std]
#![feature(used_with_arg)]

use ariel_os::debug::{log::*, ExitCode};
use ariel_os::thread::sync::Channel;

static CHANNEL: Channel<u8> = Channel::new();

#[ariel_os::thread(autostart)]
fn thread0() {
    let my_id = ariel_os::thread::current_pid().unwrap();
    info!("[Thread {}] Sending a message...", my_id);
    CHANNEL.send(&42);
}

#[ariel_os::thread(autostart, stacksize = 4096, priority = 2)]
fn thread1() {
    let my_id = ariel_os::thread::current_pid().unwrap();
    info!("[Thread {}] Waiting to receive a message...", my_id);
    let recv = CHANNEL.recv();
    info!(
        "[Thread {}] The answer to the Ultimate Question of Life, the Universe, and Everything is: {}.",
        my_id,
        recv
    );
    ariel_os::debug::exit(ExitCode::SUCCESS);
}
