pub mod gpio;

#[cfg(feature = "usb")]
pub mod usb;

use embassy_rp::config::Config;

pub(crate) use embassy_executor::InterruptExecutor as Executor;
pub use embassy_rp::interrupt;
pub use embassy_rp::{peripherals, OptionalPeripherals};

crate::executor_swi!(SWI_IRQ_1);

pub fn init() -> OptionalPeripherals {
    // SWI & DMA priority need to match. DMA is hard-coded to P3 by upstream.
    use embassy_rp::interrupt::{InterruptExt, Priority};
    SWI.set_priority(Priority::P3);

    let peripherals = embassy_rp::init(Config::default());
    OptionalPeripherals::from(peripherals)
}
