#![no_std]
#![feature(doc_auto_cfg)]

pub mod gpio;

pub mod peripheral {
    pub use embassy_nrf::Peripheral;
}

#[cfg(feature = "external-interrupts")]
pub mod extint_registry;

#[cfg(feature = "hwrng")]
pub mod hwrng;

#[cfg(feature = "i2c")]
pub mod i2c;

pub mod identity;

#[cfg(feature = "spi")]
pub mod spi;

#[cfg(feature = "storage")]
pub mod storage;

#[cfg(feature = "usb")]
pub mod usb;

#[cfg(feature = "executor-interrupt")]
pub use embassy_executor::InterruptExecutor as Executor;

#[cfg(feature = "executor-interrupt")]
#[cfg(context = "nrf52")]
ariel_os_embassy_common::executor_swi!(SWI0_EGU0);

#[cfg(feature = "executor-interrupt")]
#[cfg(context = "nrf5340")]
ariel_os_embassy_common::executor_swi!(EGU0);

use embassy_nrf::config::Config;

pub use embassy_nrf::{interrupt, peripherals, OptionalPeripherals};

#[cfg(feature = "executor-interrupt")]
pub static EXECUTOR: Executor = Executor::new();

pub fn init() -> OptionalPeripherals {
    let peripherals = embassy_nrf::init(Config::default());
    OptionalPeripherals::from(peripherals)
}
