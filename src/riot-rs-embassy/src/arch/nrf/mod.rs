pub mod gpio;

#[cfg(feature = "i2c")]
pub mod i2c;
#[cfg(feature = "spi")]
pub mod spi;

pub mod peripheral {
    pub use embassy_nrf::Peripheral;
}

#[cfg(feature = "hwrng")]
pub mod hwrng;

#[cfg(feature = "usb")]
pub mod usb;

#[cfg(feature = "executor-interrupt")]
pub(crate) use embassy_executor::InterruptExecutor as Executor;

#[cfg(feature = "executor-interrupt")]
#[cfg(context = "nrf52")]
crate::executor_swi!(SWI0_EGU0);

#[cfg(feature = "executor-interrupt")]
#[cfg(context = "nrf5340")]
crate::executor_swi!(EGU0);

use embassy_nrf::config::Config;

pub use embassy_nrf::{interrupt, peripherals, OptionalPeripherals};

pub fn init() -> OptionalPeripherals {
    let peripherals = embassy_nrf::init(Config::default());
    OptionalPeripherals::from(peripherals)
}
