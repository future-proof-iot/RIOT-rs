use embassy_rp::{
    bind_interrupts, peripherals,
    usb::{Driver, InterruptHandler},
};

use crate::arch;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<peripherals::USB>;
});

pub type UsbDriver = Driver<'static, peripherals::USB>;

pub fn driver(peripherals: &mut arch::OptionalPeripherals) -> UsbDriver {
    let usb = peripherals.USB.take().unwrap();
    Driver::new(usb, Irqs)
}
