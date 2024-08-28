use embassy_rp::{
    bind_interrupts, peripherals,
    usb::{Driver, InterruptHandler},
};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<peripherals::USB>;
});

pub type UsbDriver = Driver<'static, peripherals::USB>;

pub fn driver(peripherals: &mut crate::OptionalPeripherals) -> UsbDriver {
    let usb = peripherals.USB.take().unwrap();
    Driver::new(usb, Irqs)
}
