#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use riot_rs::debug::log::*;
use riot_rs::thread::channel::Channel;

static CHANNEL: Channel<u8> = Channel::new();

#[riot_rs::thread(autostart)]
fn thread0() {
    let my_id = riot_rs::thread::current_pid().unwrap();
    info!("[Thread {:?}] Sending a message...", my_id);
    CHANNEL.send(&42);
}

#[riot_rs::thread(autostart, stacksize = 4096, priority = 2)]
fn thread1() {
    let my_id = riot_rs::thread::current_pid().unwrap();
    info!("[Thread {:?}] Waiting to receive a message...", my_id);
    let recv = CHANNEL.recv();
    info!(
        "[Thread {:?}] The answer to the Ultimate Question of Life, the Universe, and Everything is: {:?}.",
        my_id,
        recv
    );
}
