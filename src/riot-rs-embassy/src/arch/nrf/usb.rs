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
