pub mod gpio;

#[cfg(feature = "usb")]
pub mod usb;

pub(crate) use embassy_executor::InterruptExecutor as Executor;
pub use embassy_rp::interrupt;
pub use embassy_rp::interrupt::SWI_IRQ_1 as SWI;
pub use embassy_rp::{config::Config, peripherals, OptionalPeripherals};

#[interrupt]
unsafe fn SWI_IRQ_1() {
    // SAFETY:
    // - called from ISR
    // - not called before `start()`, as the interrupt is enabled by `start()`
    //   itself
    unsafe { crate::EXECUTOR.on_interrupt() }
}

pub fn init(config: Config) -> OptionalPeripherals {
    // SWI & DMA priority need to match. DMA is hard-coded to P3 by upstream.
    use embassy_rp::interrupt::{InterruptExt, Priority};
    SWI.set_priority(Priority::P3);

    let peripherals = embassy_rp::init(config);
    OptionalPeripherals::from(peripherals)
}
