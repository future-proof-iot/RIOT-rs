use esp_hal::{
    peripherals,
    timer::systimer::{SystemTimer, Unit as _},
};

use crate::Error;

#[allow(missing_docs)]
pub fn benchmark<F: Fn() -> ()>(iterations: usize, f: F) -> Result<usize, Error> {
    let mut systimer_periph = unsafe { peripherals::SYSTIMER::steal() };
    let timer = SystemTimer::new(&mut systimer_periph);

    // Reset counter of unit0, which is read in `SystemTimer::now()`.
    timer.unit0.set_count(0);

    while SystemTimer::now() == 0 {}

    let before = SystemTimer::now();

    for _ in 0..iterations {
        f();
    }

    SystemTimer::now()
        .checked_sub(before)
        .map(|total| total as usize / iterations)
        .ok_or(Error::SystemTimerWrapped)
}
