pub(crate) use embassy_executor::InterruptExecutor as Executor;
pub use embassy_rp::interrupt;
pub use embassy_rp::interrupt::SWI_IRQ_1 as SWI;
pub use embassy_rp::{config::Config, peripherals, OptionalPeripherals, Peripherals};

#[cfg(feature = "usb")]
use embassy_rp::{bind_interrupts, peripherals::USB, usb::InterruptHandler};

// rp2040 usb start
#[cfg(feature = "usb")]
bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

#[interrupt]
unsafe fn SWI_IRQ_1() {
    crate::EXECUTOR.on_interrupt()
}

#[cfg(feature = "usb")]
pub mod usb {
    use embassy_rp::peripherals;
    use embassy_rp::usb::Driver;

    use crate::arch;

    pub type UsbDriver = Driver<'static, peripherals::USB>;

    pub fn driver(peripherals: &mut arch::OptionalPeripherals) -> UsbDriver {
        let usb = peripherals.USB.take().unwrap();
        Driver::new(usb, super::Irqs)
    }
}

pub fn init(config: Config) -> Peripherals {
    // SWI & DMA priority need to match. DMA is hard-coded to P3 by upstream.
    use embassy_rp::interrupt::{InterruptExt, Priority};
    SWI.set_priority(Priority::P3);

    embassy_rp::init(config)
}
