use embassy_stm32::{bind_interrupts, peripherals, usb, usb::Driver};

use crate::arch;

bind_interrupts!(struct Irqs {
    USB_LP => usb::InterruptHandler<peripherals::USB>;
});

pub type UsbDriver = Driver<'static, peripherals::USB>;

pub fn driver(peripherals: &mut arch::OptionalPeripherals) -> UsbDriver {
    let usb = peripherals.USB.take().unwrap();
    let dp = peripherals.PA12.take().unwrap();
    let dm = peripherals.PA11.take().unwrap();

    Driver::new(usb, Irqs, dp, dm)
}
