#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use embassy_executor::{InterruptExecutor, Spawner};
use embassy_nrf::interrupt;
use embassy_nrf::interrupt::{InterruptExt, Priority};

use critical_section::Mutex;
use riot_rs_rt::debug;

pub static EXECUTOR: InterruptExecutor = InterruptExecutor::new();

pub mod blocker;

#[interrupt]
unsafe fn SWI0_EGU0() {
    EXECUTOR.on_interrupt()
}

fn init() {
    debug::println!("riot-rs-embassy::init()");
    let _p = embassy_nrf::init(Default::default());
    EXECUTOR.start(interrupt::SWI0_EGU0);
}

use linkme::distributed_slice;
use riot_rs_rt::INIT_FUNCS;

#[distributed_slice(INIT_FUNCS)]
static RIOT_RS_EMBASSY_INIT: fn() = init;
