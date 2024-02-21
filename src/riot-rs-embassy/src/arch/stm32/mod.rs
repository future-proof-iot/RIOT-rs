pub(crate) use embassy_executor::InterruptExecutor as Executor;
pub use embassy_stm32::interrupt;
pub use embassy_stm32::interrupt::UART5 as SWI;
pub use embassy_stm32::{peripherals, Config, OptionalPeripherals, Peripherals};

#[interrupt]
unsafe fn UART5() {
    // SAFETY:
    // - called from ISR
    // - not called before `start()`, as the interrupt is enabled by `start()`
    //   itself
    unsafe { crate::EXECUTOR.on_interrupt() }
}

pub fn init(config: Config) -> OptionalPeripherals {
    let peripherals = embassy_stm32::init(config);
    OptionalPeripherals::from(peripherals)
}
