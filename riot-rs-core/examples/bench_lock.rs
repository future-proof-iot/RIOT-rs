#![no_main]
#![no_std]
use riot_rs_core::thread::{CreateFlags, Lock, Thread};

extern crate cortex_m;
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m::peripheral::Peripherals;

use riot_rs_rt::debug::println;

#[allow(non_snake_case)]
#[no_mangle]
fn SysTick() {
    println!("systick");
    Thread::wakeup(2);
}

static mut STACK: [u8; 1024] = [0; 1024];

static LOCK: Lock = Lock::new();

fn func(_arg: usize) {
    loop {
        LOCK.acquire();
    }
}

#[no_mangle]
fn user_main() {
    let mut p = Peripherals::take().unwrap();
    //
    p.SCB.clear_sleepdeep();

    //
    p.SYST.set_clock_source(SystClkSource::Core);
    p.SYST.set_reload(8_000_000);
    p.SYST.clear_current();
    p.SYST.enable_counter();
    //p.SYST.enable_interrupt();

    unsafe {
        Thread::create(&mut STACK, func, 0, 6, CreateFlags::empty());
    }

    LOCK.acquire();
    Thread::yield_higher();

    const N: usize = 1000;

    while cortex_m::peripheral::SYST::get_current() == 0 {}

    let before = cortex_m::peripheral::SYST::get_current();

    for _ in 0..N {
        LOCK.release();
    }

    let total = before - cortex_m::peripheral::SYST::get_current();

    assert!(!p.SYST.has_wrapped());

    println!("total: {} ticks: {}", total, total as usize / N);
}
