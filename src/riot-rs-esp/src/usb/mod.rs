use esp_hal::otg_fs::asynch::Driver;
use static_cell::ConstStaticCell;

use crate::peripherals;

pub type UsbDriver = Driver<'static>;

pub struct Peripherals {
    usb: peripherals::USB0,
    usbdp: peripherals::GPIO_20,
    usbdm: peripherals::GPIO_19,
}

impl Peripherals {
    #[must_use]
    pub fn new(peripherals: &mut crate::OptionalPeripherals) -> Self {
        Self {
            usb: peripherals.USB0.take().unwrap(),
            usbdp: peripherals.GPIO_20.take().unwrap(),
            usbdm: peripherals.GPIO_19.take().unwrap(),
        }
    }
}
pub fn driver(peripherals: Peripherals) -> UsbDriver {
    use esp_hal::otg_fs::{asynch::Config, Usb};

    let usb = Usb::new(peripherals.usb, peripherals.usbdp, peripherals.usbdm);

    // Buffer size copied from upstream. There's no hint about sizing.
    static EP_OUT_BUFFER: ConstStaticCell<[u8; 1024]> = ConstStaticCell::new([0u8; 1024]);
    let ep_out_buffer = EP_OUT_BUFFER.take();

    let config = Config::default();

    Driver::new(usb, ep_out_buffer, config)
}
