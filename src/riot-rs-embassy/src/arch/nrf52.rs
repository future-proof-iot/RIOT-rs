pub(crate) use embassy_executor::InterruptExecutor as Executor;

#[cfg(context = "nrf52")]
pub use embassy_nrf::interrupt::SWI0_EGU0 as SWI;

#[cfg(context = "nrf5340")]
pub use embassy_nrf::interrupt::EGU0 as SWI;

pub use embassy_nrf::{config::Config, interrupt, peripherals, OptionalPeripherals};

#[cfg(context = "nrf52")]
#[interrupt]
unsafe fn SWI0_EGU0() {
    // SAFETY:
    // - called from ISR
    // - not called before `start()`, as the interrupt is enabled by `start()`
    //   itself
    unsafe { crate::EXECUTOR.on_interrupt() }
}

#[cfg(context = "nrf5340")]
#[interrupt]
unsafe fn EGU0() {
    unsafe { crate::EXECUTOR.on_interrupt() }
}

#[cfg(feature = "usb")]
pub mod usb {
    use embassy_nrf::{
        bind_interrupts, peripherals,
        usb::{
            self,
            vbus_detect::{self, HardwareVbusDetect},
            Driver,
        },
    };

    use crate::arch;

    #[cfg(context = "nrf52")]
    bind_interrupts!(struct Irqs {
        USBD => usb::InterruptHandler<peripherals::USBD>;
        POWER_CLOCK => vbus_detect::InterruptHandler;
    });

    #[cfg(context = "nrf5340")]
    bind_interrupts!(struct Irqs {
        USBD => usb::InterruptHandler<peripherals::USBD>;
        USBREGULATOR => vbus_detect::InterruptHandler;
    });

    pub type UsbDriver = Driver<'static, peripherals::USBD, HardwareVbusDetect>;

    pub fn driver(peripherals: &mut arch::OptionalPeripherals) -> UsbDriver {
        let usbd = peripherals.USBD.take().unwrap();
        Driver::new(usbd, Irqs, HardwareVbusDetect::new(Irqs))
    }
}

#[cfg(feature = "hwrng")]
pub mod hwrng {
    embassy_nrf::bind_interrupts!(pub struct Irqs {
        RNG => embassy_nrf::rng::InterruptHandler<embassy_nrf::peripherals::RNG>;
    });
}

pub fn init(config: Config) -> OptionalPeripherals {
    let peripherals = embassy_nrf::init(config);
    OptionalPeripherals::from(peripherals)
}
