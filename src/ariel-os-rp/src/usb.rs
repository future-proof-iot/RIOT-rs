use embassy_rp::{
    bind_interrupts, peripherals,
    usb::{Driver, InterruptHandler},
};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<peripherals::USB>;
});

pub type UsbDriver = Driver<'static, peripherals::USB>;

pub struct Peripherals {
    usb: peripherals::USB,
}

impl Peripherals {
    #[must_use]
    pub fn new(peripherals: &mut crate::OptionalPeripherals) -> Self {
        Self {
            usb: peripherals.USB.take().unwrap(),
        }
    }
}

pub fn driver(peripherals: Peripherals) -> UsbDriver {
    Driver::new(peripherals.usb, Irqs)
}
