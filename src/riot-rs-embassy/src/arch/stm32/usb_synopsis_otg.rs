use embassy_stm32::{
    bind_interrupts, peripherals, usb,
    usb::{Driver, InterruptHandler},
};

use crate::arch;

bind_interrupts!(struct Irqs {
    OTG_FS => usb::InterruptHandler<peripherals::USB_OTG_FS>;
});

pub type UsbDriver = Driver<'static, peripherals::USB_OTG_FS>;

pub fn driver(peripherals: &mut arch::OptionalPeripherals) -> UsbDriver {
    let usb = peripherals.USB.take().unwrap();
    let dp = peripherals.PA12.take().unwrap();
    let dm = peripherals.PA12.take().unwrap();

    let mut ep_out_buffer = crate::make_static!([0u8; 256]);
    let mut config = embassy_stm32::usb::Config::default();

    // Enable vbus_detection
    // Note: some boards don't have this wired up and might not require it,
    // as they are powered through usb!
    // If you hang on boot, try setting this to "false"!
    // See https://embassy.dev/book/dev/faq.html#_the_usb_examples_are_not_working_on_my_board_is_there_anything_else_i_need_to_configure
    // for more information
    config.vbus_detection = true;

    Driver::new_fs(usb, Irqs, dp, dm, &mut ep_out_buffer, config)
}
