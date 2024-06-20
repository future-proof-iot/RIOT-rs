pub mod gpio;

pub mod peripheral {
    pub use embassy_rp::Peripheral;
}

#[cfg(feature = "usb")]
pub mod usb;

pub use embassy_rp::{peripherals, OptionalPeripherals};

#[cfg(feature = "executor-interrupt")]
pub(crate) use embassy_executor::InterruptExecutor as Executor;
#[cfg(feature = "executor-interrupt")]
pub use embassy_rp::interrupt;

#[cfg(feature = "executor-interrupt")]
crate::executor_swi!(SWI_IRQ_1);

pub fn init() -> OptionalPeripherals {
    #[cfg(feature = "executor-interrupt")]
    {
        // SWI & DMA priority need to match. DMA is hard-coded to P3 by upstream.
        use embassy_rp::interrupt::{InterruptExt as _, Priority};
        SWI.set_priority(Priority::P3);
    }

    let peripherals = embassy_rp::init(embassy_rp::config::Config::default());
    OptionalPeripherals::from(peripherals)
}
