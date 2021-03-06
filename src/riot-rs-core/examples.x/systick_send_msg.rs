#![no_main]
#![no_std]
use riot_rs_core::thread::{CreateFlags, Msg, Thread};

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

fn func(_arg: usize) {
    loop {
        println!("func()");
        unsafe {
            Thread::send_msg(
                Msg {
                    a: 1,
                    b: 2,
                    c: 3,
                    d: 4,
                },
                Thread::get_mut(1),
            );
        }
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
        let m = Thread::current().receive_msg();
        println!("{:#?}", m);
    }
}
