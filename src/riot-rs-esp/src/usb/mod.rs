use esp_hal::otg_fs::asynch::Driver;
use static_cell::ConstStaticCell;

pub type UsbDriver = Driver<'static>;

pub fn driver(peripherals: &mut crate::OptionalPeripherals) -> UsbDriver {
    use esp_hal::otg_fs::{asynch::Config, Usb};

    let usb = Usb::new(
        peripherals.USB0.take().unwrap(),
        peripherals.GPIO_20.take().unwrap(),
        peripherals.GPIO_19.take().unwrap(),
    );

    // Buffer size copied from upstream. There's no hint about sizing.
    static EP_OUT_BUFFER: ConstStaticCell<[u8; 1024]> = ConstStaticCell::new([0u8; 1024]);
    let ep_out_buffer = EP_OUT_BUFFER.take();

    let config = Config::default();

    Driver::new(usb, ep_out_buffer, config)
}
