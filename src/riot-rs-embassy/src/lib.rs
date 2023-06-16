#![no_std]
#![feature(type_alias_impl_trait)]

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

#[no_mangle]
extern "C" fn riot_rs_embassy_init() {
    let _p = embassy_nrf::init(Default::default());
    EXECUTOR.start(interrupt::SWI0_EGU0);
}
