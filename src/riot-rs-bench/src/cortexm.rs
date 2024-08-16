use cortex_m::{
    peripheral::{syst::SystClkSource, SYST},
    Peripherals,
};

use crate::Error;

#[allow(missing_docs)]
pub fn benchmark<F: Fn() -> ()>(iterations: usize, f: F) -> Result<usize, Error> {
    let mut p = unsafe { Peripherals::steal() };
    //
    p.SCB.clear_sleepdeep();

    //
    p.SYST.set_clock_source(SystClkSource::Core);
    p.SYST.set_reload(0x00FFFFFF);
    p.SYST.clear_current();
    p.SYST.enable_counter();

    // Wait for the system timer to be ready
    while SYST::get_current() == 0 {}

    let before = SYST::get_current();

    for _ in 0..iterations {
        f();
    }

    // SysTick is downcounting, so `before - after` is correct.
    let total = before - SYST::get_current();

    if p.SYST.has_wrapped() {
        Err(Error::SystemTimerWrapped)
    } else {
        Ok(total as usize / iterations)
    }
}
