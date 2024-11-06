use embassy_nrf::{
    bind_interrupts, pac, peripherals,
    usb::{
        self,
        vbus_detect::{self, HardwareVbusDetect},
        Driver,
    },
};
use riot_rs_debug::log::debug;

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

pub fn init() {
    debug!("nrf: enabling ext hfosc...");
    pac::CLOCK.tasks_hfclkstart().write_value(1);
    while pac::CLOCK.events_hfclkstarted().read() != 1 {}
}

pub fn driver(peripherals: &mut crate::OptionalPeripherals) -> UsbDriver {
    let usbd = peripherals.USBD.take().unwrap();
    Driver::new(usbd, Irqs, HardwareVbusDetect::new(Irqs))
}
