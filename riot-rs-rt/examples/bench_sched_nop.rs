#![no_main]
#![no_std]
use riot_core::thread::Thread;

extern crate cortex_m;
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m::peripheral::Peripherals;
//use cortex_m::peripheral::SCB;

use riot_core::testing::println;

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
    while cortex_m::peripheral::SYST::get_current() == 0 {}

    const N: usize = 100_000;

    let before = cortex_m::peripheral::SYST::get_current();

    for _ in 0..N {
        Thread::yield_higher();
    }

    assert!(!p.SYST.has_wrapped());

    let total = before - cortex_m::peripheral::SYST::get_current();
    println!("total: {} ticks: {}", total, total as usize / N);
}
