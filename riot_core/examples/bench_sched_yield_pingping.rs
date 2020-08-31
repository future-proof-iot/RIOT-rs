#![no_main]
#![no_std]
use riot_core::thread::Thread;

extern crate cortex_m;
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m::peripheral::Peripherals;
//use cortex_m::peripheral::SCB;

use riot_core::testing::println;

static mut STACK: [u8; 1024] = [0; 1024];

fn func(arg: usize) {
    loop {
        Thread::yield_next();
    }
}

#[no_mangle]
fn user_main() {
    let mut p = Peripherals::take().unwrap();
    //
    p.SCB.clear_sleepdeep();

    //
    p.SYST.set_clock_source(SystClkSource::Core);
    p.SYST.set_reload(0x00FFFFFF);
    p.SYST.clear_current();
    p.SYST.enable_counter();
    //p.SYST.disable_interrupt();

    unsafe {
        Thread::create(&mut STACK, func, 0, 5);
    }

    const N: usize = 1000;

    let before = cortex_m::peripheral::SYST::get_current();
    let mut count = N;
    loop {
        Thread::yield_next();
        count -= 1;
        if count == 0 {
            break;
        }
    }
    let total = before - cortex_m::peripheral::SYST::get_current();
    println!("total: {} ticks: {}", total, total as usize / (2 * N)).unwrap();
}
