#![no_main]
#![no_std]
use riot_rs_core::thread::{CreateFlags, Thread};

extern crate cortex_m;
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m::peripheral::Peripherals;
use cortex_m::peripheral::SCB;

use riot_rs_rt::debug::println;

#[allow(non_snake_case)]
#[no_mangle]
fn SysTick() {
    println!("systick");
    Thread::wakeup(2);
}

static mut STACK: [u8; 1024] = [0; 1024];

fn func(_arg: usize) {
    loop {
        println!("func()");
        Thread::sleep();
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
    p.SYST.enable_interrupt();

    unsafe {
        Thread::create(&mut STACK, func, 0, 5, CreateFlags::empty());
    }

    loop {
        SCB::set_pendsv();
        cortex_m::asm::isb();
    }
}
