#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

mod pins;

use core::cell::Cell;

use riot_rs::embassy::{
    arch, usbd_hid::descriptor::KeyboardReport, Application, ApplicationInitError, Drivers,
    InitializationArgs, UsbHidWriter,
};
use riot_rs::rt::debug::println;

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embassy_time::Duration;
use static_cell::make_static;

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

#[embassy_executor::task]
async fn usb_keyboard(
    mut buttons: Buttons,
    hid_writer: &'static Mutex<CriticalSectionRawMutex, UsbHidWriter>,
) {
    loop {
        for (i, button) in buttons.get_mut().iter_mut().enumerate() {
            if button.is_pressed() {
                println!("Button #{} pressed", i + 1);

                let report = keyboard_report(KEYCODE_MAPPING[i]);
                if let Err(e) = hid_writer.lock().await.write_serialize(&report).await {
                    println!("Failed to send report: {:?}", e);
                }
                let report = keyboard_report(KEY_RELEASED);
                if let Err(e) = hid_writer.lock().await.write_serialize(&report).await {
                    println!("Failed to send report: {:?}", e);
                }
            }
        }

        // Debounce events
        embassy_time::Timer::after(Duration::from_millis(50)).await;
    }
}

struct UsbKeyboard {
    buttons: Cell<Option<Buttons>>,
}

impl Application for UsbKeyboard {
    fn initialize(
        peripherals: &mut arch::OptionalPeripherals,
        _init_args: InitializationArgs,
    ) -> Result<&dyn Application, ApplicationInitError> {
        let our_peripherals = pins::OurPeripherals::take_from(peripherals)?;

        let buttons = Buttons::new(our_peripherals.buttons);

        Ok(make_static!(Self {
            buttons: Cell::new(Some(buttons)),
        }))
    }

    fn start(&self, spawner: embassy_executor::Spawner, drivers: Drivers) {
        let hid_writer = drivers.usb_hid.get().unwrap().writer;
        spawner
            .spawn(usb_keyboard(self.buttons.take().unwrap(), hid_writer))
            .unwrap();
    }
}

riot_rs::embassy::riot_initialize!(UsbKeyboard);

mod buttons {
    use embassy_nrf::gpio::{AnyPin, Input, Pin, Pull};

    use crate::pins;

    pub const KEY_COUNT: u8 = 4;

    pub struct Button(Input<'static, AnyPin>);

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
