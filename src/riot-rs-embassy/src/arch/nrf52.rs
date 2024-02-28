pub(crate) use embassy_executor::InterruptExecutor as Executor;
pub use embassy_nrf::interrupt::SWI0_EGU0 as SWI;
pub use embassy_nrf::{config::Config, interrupt, peripherals, OptionalPeripherals};

#[interrupt]
unsafe fn SWI0_EGU0() {
    crate::EXECUTOR.on_interrupt()
}

#[cfg(feature = "usb")]
pub mod usb {
    use embassy_nrf::{
        bind_interrupts, peripherals, rng,
        usb::{
            self,
            vbus_detect::{self, HardwareVbusDetect},
            Driver,
        },
    };

    use crate::arch;

    bind_interrupts!(struct Irqs {
        USBD => usb::InterruptHandler<peripherals::USBD>;
        POWER_CLOCK => vbus_detect::InterruptHandler;
        RNG => rng::InterruptHandler<peripherals::RNG>;
    });

    pub type UsbDriver = Driver<'static, peripherals::USBD, HardwareVbusDetect>;

    pub fn driver(peripherals: &mut arch::OptionalPeripherals) -> UsbDriver {
        let usbd = peripherals.USBD.take().unwrap();
        Driver::new(usbd, Irqs, HardwareVbusDetect::new(Irqs))
    }
}

pub fn init(config: Config) -> OptionalPeripherals {
    let peripherals = embassy_nrf::init(config);
    OptionalPeripherals::from(peripherals)
}
