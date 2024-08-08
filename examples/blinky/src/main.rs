#![no_main]
#![no_std]
#![feature(impl_trait_in_assoc_type)]
#![feature(used_with_arg)]

mod pins;

use embassy_time::{Duration, Timer};
use riot_rs::gpio::{Level, Output};

// ANCHOR: task-example-0
#[riot_rs::task(autostart, peripherals)]
async fn blinky(peripherals: pins::LedPeripherals) {
    let mut led = Output::new(peripherals.led, Level::Low);

    // ANCHOR_END: task-example-0
    // The micro:bit uses an LED matrix; pull the column line low.
    #[cfg(context = "microbit-v2")]
    let _led_col1 = Output::new(peripherals.led_col1, Level::Low);

    // ANCHOR: task-example-1
    loop {
        led.toggle();
        Timer::after(Duration::from_millis(500)).await;
    }
}
// ANCHOR_END: task-example-1
