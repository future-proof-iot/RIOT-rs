use esp_hal::{
    clock::ClockControl,
    embassy::{
        self,
        executor::{FromCpu1, InterruptExecutor},
    },
    prelude::*,
};

pub(crate) use esp_hal::interrupt::{self};
pub use esp_hal::peripherals::{OptionalPeripherals, Peripherals};

pub(crate) type Executor = InterruptExecutor<FromCpu1>;
pub static SWI: () = ();

#[derive(Default)]
pub struct Config {}

#[interrupt]
fn FROM_CPU_INTR1() {
    unsafe { crate::EXECUTOR.on_interrupt() }
}

pub fn init(_config: Config) -> OptionalPeripherals {
    let mut peripherals = OptionalPeripherals::from(Peripherals::take());
    let system = peripherals.SYSTEM.take().unwrap().split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    embassy::init(
        &clocks,
        esp_hal::systimer::SystemTimer::new(peripherals.SYSTIMER.take().unwrap()),
    );

    peripherals
}
