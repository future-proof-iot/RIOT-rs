#![no_main]
#![no_std]
use riot_rs_sched::thread::{Lock, Thread};

extern crate cortex_m;
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m::peripheral::Peripherals;

use riot_rs_rt::debug::println;

static mut STACK: [u8; 1024] = [0; 1024];

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

    let lock = Lock::new();
    let val = 0x1020304050607080u64;
    let val2 = 0x1122334455667788u64;

    unsafe {
        Thread::spawn(&mut STACK, move || {
            println!("from closure: {:x} {:x}", val, val2);
        });
    };

    lock.acquire();
    Thread::yield_higher();

    const N: usize = 1000;

    while cortex_m::peripheral::SYST::get_current() == 0 {}

    let before = cortex_m::peripheral::SYST::get_current();

    for _ in 0..N {
        lock.release();
    }

    let total = before - cortex_m::peripheral::SYST::get_current();

    assert!(!p.SYST.has_wrapped());

    println!("total: {} ticks: {}", total, total as usize / N);
}
