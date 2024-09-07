#![no_main]
#![no_std]
#![feature(impl_trait_in_assoc_type)]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

mod pins;

use embassy_time::{Duration, Timer};
use riot_rs::gpio::{Input, Level, Output, Pull};

#[riot_rs::task(autostart, peripherals)]
async fn blinky(peripherals: pins::Peripherals) {
    let mut led1 = Output::new(peripherals.led1, Level::Low);

    #[allow(unused_variables)]
    let pull = Pull::Up;
    #[cfg(context = "st-nucleo-h755zi-q")]
    let pull = Pull::None;

    let mut btn1 = Input::builder(peripherals.btn1, pull)
        .build_with_interrupt()
        .unwrap();

    // The micro:bit uses an LED matrix; pull the column line low.
    #[cfg(context = "microbit-v2")]
    let _led_col1 = Output::new(peripherals.led_col1, Level::Low);

    loop {
        // Wait for the button being pressed or 300 ms, whichever comes first.
        let _ = embassy_futures::select::select(
            btn1.wait_for_low(),
            Timer::after(Duration::from_millis(300)),
        )
        .await;

        led1.toggle();
        Timer::after(Duration::from_millis(100)).await;
    }
}
