pub use embassy_rp::interrupt;
pub use embassy_rp::interrupt::SWI_IRQ_1 as SWI;
pub use embassy_rp::{init, OptionalPeripherals, Peripherals};

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
    pub type UsbDriver = Driver<'static, peripherals::USB>;
    pub fn driver(usb: peripherals::USB) -> UsbDriver {
        Driver::new(usb, super::Irqs)
    }
}
