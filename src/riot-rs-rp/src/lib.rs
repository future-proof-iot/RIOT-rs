#![no_std]
#![feature(doc_auto_cfg)]
#![feature(impl_trait_in_assoc_type)]
#![feature(type_alias_impl_trait)]
#![feature(lint_reasons)]

pub mod gpio;

pub mod peripheral {
    pub use embassy_rp::Peripheral;
}

#[cfg(feature = "wifi")]
mod wifi;

#[cfg(feature = "wifi-cyw43")]
pub mod cyw43;

#[cfg(feature = "hwrng")]
pub mod hwrng;

#[cfg(feature = "usb")]
pub mod usb;

pub use embassy_rp::{peripherals, OptionalPeripherals};

#[cfg(feature = "executor-interrupt")]
pub use embassy_executor::InterruptExecutor as Executor;
#[cfg(feature = "executor-interrupt")]
pub use embassy_rp::interrupt;

#[cfg(feature = "executor-interrupt")]
riot_rs_embassy_common::executor_swi!(SWI_IRQ_1);

#[cfg(feature = "executor-interrupt")]
pub static EXECUTOR: Executor = Executor::new();

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
