use embassy_stm32::{bind_interrupts, peripherals, usb, usb::Driver};
use static_cell::ConstStaticCell;

bind_interrupts!(struct Irqs {
    OTG_FS => usb::InterruptHandler<peripherals::USB_OTG_FS>;
});

pub type UsbDriver = Driver<'static, peripherals::USB_OTG_FS>;

pub struct Peripherals {
    usb: peripherals::USB_OTG_FS,
    dp: peripherals::PA12,
    dm: peripherals::PA11,
}

impl Peripherals {
    #[must_use]
    pub fn new(peripherals: &mut crate::OptionalPeripherals) -> Self {
        Self {
            usb: peripherals.USB_OTG_FS.take().unwrap(),
            dp: peripherals.PA12.take().unwrap(),
            dm: peripherals.PA11.take().unwrap(),
        }
    }
}

pub fn driver(peripherals: Peripherals) -> UsbDriver {
    // buffer size copied from upstream. There's this hint about its sizing:
    // "An internal buffer used to temporarily store received packets.
    // Must be large enough to fit all OUT endpoint max packet sizes.
    // Endpoint allocation will fail if it is too small."
    static EP_OUT_BUFFER: ConstStaticCell<[u8; 256]> = ConstStaticCell::new([0u8; 256]);
    let ep_out_buffer = EP_OUT_BUFFER.take();
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
        crate::SWI.set_priority(Priority::P1);
        embassy_stm32::interrupt::OTG_FS.set_priority(Priority::P0);
    }

    Driver::new_fs(
        peripherals.usb,
        Irqs,
        peripherals.dp,
        peripherals.dm,
        ep_out_buffer,
        config,
    )
}
