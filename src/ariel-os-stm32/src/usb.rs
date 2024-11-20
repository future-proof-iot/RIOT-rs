use embassy_stm32::{bind_interrupts, peripherals, usb, usb::Driver};

bind_interrupts!(struct Irqs {
    USB_LP => usb::InterruptHandler<peripherals::USB>;
});

pub type UsbDriver = Driver<'static, peripherals::USB>;

pub struct Peripherals {
    usb: peripherals::USB,
    dp: peripherals::PA12,
    dm: peripherals::PA11,
}

impl Peripherals {
    #[must_use]
    pub fn new(peripherals: &mut crate::OptionalPeripherals) -> Self {
        Self {
            usb: peripherals.USB.take().unwrap(),
            dp: peripherals.PA12.take().unwrap(),
            dm: peripherals.PA11.take().unwrap(),
        }
    }
}

pub fn driver(peripherals: Peripherals) -> UsbDriver {
    Driver::new(peripherals.usb, Irqs, peripherals.dp, peripherals.dm)
}
