#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]
#![feature(used_with_arg)]

use embassy_time::Duration;
use embassy_usb::class::hid::{self, HidReaderWriter};
use riot_rs::{
    debug::log::*,
    usb::{UsbBuilderHook, UsbDriver},
    ConstStaticCell,
};

use usbd_hid::descriptor::KeyboardReport;

mod pins;

#[riot_rs::task(autostart, peripherals, usb_builder_hook)]
async fn usb_keyboard(button_peripherals: pins::Buttons) {
    let mut buttons = Buttons::new(button_peripherals);

    let config = embassy_usb::class::hid::Config {
            report_descriptor: <usbd_hid::descriptor::KeyboardReport as usbd_hid::descriptor::SerializedDescriptor>::desc(),
            request_handler: None,
            poll_ms: 60,
            max_packet_size: 64,
        };

    static HID_STATE: ConstStaticCell<hid::State> = ConstStaticCell::new(hid::State::new());
    let hid_state = HID_STATE.take();

    let hid_rw: HidReaderWriter<'static, UsbDriver, 1, 8> = USB_BUILDER_HOOK
        .with(|usb_builder| hid::HidReaderWriter::new(usb_builder, hid_state, config))
        .await;

    let (_hid_reader, mut hid_writer) = hid_rw.split();

    loop {
        for (i, button) in buttons.get_mut().iter_mut().enumerate() {
            if button.is_pressed() {
                info!("Button #{} pressed", i + 1);

                let report = keyboard_report(KEYCODE_MAPPING[i]);
                if let Err(e) = hid_writer.write_serialize(&report).await {
                    info!("Failed to send report: {:?}", e);
                }
                let report = keyboard_report(KEY_RELEASED);
                if let Err(e) = hid_writer.write_serialize(&report).await {
                    info!("Failed to send report: {:?}", e);
                }
            }
        }

        // Debounce events
        embassy_time::Timer::after(Duration::from_millis(50)).await;
    }
}

use crate::buttons::{Buttons, KEY_COUNT};

// Assuming a QWERTY US layout, see https://docs.qmk.fm/#/how_keyboards_work
// and https://www.usb.org/sites/default/files/documents/hut1_12v2.pdf
const KC_A: u8 = 0x04;
const KC_C: u8 = 0x06;
const KC_G: u8 = 0x0a;
const KC_T: u8 = 0x17;

const KEY_RELEASED: u8 = 0x00;

fn keyboard_report(keycode: u8) -> KeyboardReport {
    KeyboardReport {
        keycodes: [keycode, 0, 0, 0, 0, 0],
        leds: 0,
        modifier: 0,
        reserved: 0,
    }
}

// Maps physical buttons to keycodes/characters
const KEYCODE_MAPPING: [u8; KEY_COUNT as usize] = [KC_A, KC_C, KC_G, KC_T];

mod buttons {
    use embassy_nrf::gpio::{AnyPin, Input, Pin, Pull};

    use crate::pins;

    pub const KEY_COUNT: u8 = 4;

    pub struct Button(Input<'static>);

    impl Button {
        pub fn new(button: AnyPin) -> Self {
            Self(Input::new(button, Pull::Up))
        }

        pub fn is_pressed(&mut self) -> bool {
            self.0.is_low()
        }
    }

    pub struct Buttons([Button; KEY_COUNT as usize]);

    impl Buttons {
        pub fn new(button_peripherals: pins::Buttons) -> Self {
            Self([
                Button::new(button_peripherals.btn1.degrade()),
                Button::new(button_peripherals.btn2.degrade()),
                Button::new(button_peripherals.btn3.degrade()),
                Button::new(button_peripherals.btn4.degrade()),
            ])
        }

        pub fn get(&self) -> &[Button] {
            &self.0
        }

        pub fn get_mut(&mut self) -> &mut [Button] {
            &mut self.0
        }
    }
}

#[riot_rs::config(usb)]
fn usb_config() -> riot_rs::reexports::embassy_usb::Config<'static> {
    let mut config = riot_rs::reexports::embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("HID keyboard example");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    // Required for Windows support.
    config.composite_with_iads = true;
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config
}
