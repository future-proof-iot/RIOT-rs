use embassy_rp::{
    bind_interrupts, peripherals,
    usb::{Driver, InterruptHandler},
};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<peripherals::USB>;
});

pub type UsbDriver = Driver<'static, peripherals::USB>;

crate::define_peripherals!(Peripherals { usb: USB });

pub fn driver(peripherals: Peripherals) -> UsbDriver {
    Driver::new(peripherals.usb, Irqs)
}
