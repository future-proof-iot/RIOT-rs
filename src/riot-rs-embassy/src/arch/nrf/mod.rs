pub mod gpio;

#[cfg(feature = "hwrng")]
pub mod hwrng;

#[cfg(feature = "usb")]
pub mod usb;

pub(crate) use embassy_executor::InterruptExecutor as Executor;

#[cfg(context = "nrf52")]
crate::executor_swi!(SWI0_EGU0);

#[cfg(context = "nrf5340")]
crate::executor_swi!(EGU0);

pub use embassy_nrf::{config::Config, interrupt, peripherals, OptionalPeripherals};

pub fn init(config: Config) -> OptionalPeripherals {
    let peripherals = embassy_nrf::init(config);
    OptionalPeripherals::from(peripherals)
}
