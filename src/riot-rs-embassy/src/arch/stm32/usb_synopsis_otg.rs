use embassy_stm32::{bind_interrupts, peripherals, usb, usb::Driver};

use crate::arch;

bind_interrupts!(struct Irqs {
    OTG_FS => usb::InterruptHandler<peripherals::USB_OTG_FS>;
});

pub type UsbDriver = Driver<'static, peripherals::USB_OTG_FS>;

pub fn driver(peripherals: &mut arch::OptionalPeripherals) -> UsbDriver {
    let usb = peripherals.USB_OTG_FS.take().unwrap();
    let dp = peripherals.PA12.take().unwrap();
    let dm = peripherals.PA11.take().unwrap();

    // buffer size copied from upstream. There's this hint about its sizing:
    // "An internal buffer used to temporarily store received packets.
    // Must be large enough to fit all OUT endpoint max packet sizes.
    // Endpoint allocation will fail if it is too small."
    let ep_out_buffer = crate::make_static!([0u8; 256]);
    let mut config = embassy_stm32::usb::Config::default();

    // Enable vbus_detection
    // Note: some boards don't have this wired up and might not require it,
    // as they are powered through usb!
    // If you hang on boot, try setting this to "false"!
    // See https://embassy.dev/book/dev/faq.html#_the_usb_examples_are_not_working_on_my_board_is_there_anything_else_i_need_to_configure
    // for more information
    // NOTE(board-config)
    config.vbus_detection = true;

    #[cfg(feature = "executor-interrupt")]
    {
        use embassy_stm32::interrupt::{InterruptExt, Priority};
        crate::arch::SWI.set_priority(Priority::P1);
        embassy_stm32::interrupt::OTG_FS.set_priority(Priority::P0);
    }

    Driver::new_fs(usb, Irqs, dp, dm, ep_out_buffer, config)
}
