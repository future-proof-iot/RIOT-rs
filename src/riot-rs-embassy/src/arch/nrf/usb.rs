use embassy_nrf::{
    bind_interrupts, peripherals,
    usb::{
        self,
        vbus_detect::{self, HardwareVbusDetect},
        Driver,
    },
};

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

crate::define_peripherals!(Peripherals { usbd: USBD });

pub fn driver(peripherals: Peripherals) -> UsbDriver {
    Driver::new(peripherals.usbd, Irqs, HardwareVbusDetect::new(Irqs))
}
