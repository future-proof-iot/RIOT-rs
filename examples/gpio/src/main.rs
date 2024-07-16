#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

mod pins;

use embassy_time::{Duration, Timer};
use riot_rs::embassy::gpio::{Input, Level, Output, Pull};

#[riot_rs::task(autostart, peripherals)]
async fn blinky(peripherals: pins::Peripherals) {
    let mut led1 = Output::new(peripherals.led1, Level::Low);

    // The micro:bit uses an LED matrix; pull the column line low.
    #[cfg(context = "microbit-v2")]
    let _led_col1 = Output::new(peripherals.led_col1, Level::Low);

    loop {
        led1.toggle();
        Timer::after(Duration::from_millis(500)).await;
    }
}

#[riot_rs::task(autostart, peripherals)]
async fn button_task(peripherals: pins::ButtonPeripherals) {
    let mut btn2 = Input::builder(peripherals.btn2, Pull::Up)
        .build_with_interrupt()
        .unwrap();

    let mut led2 = Output::new(peripherals.led2, Level::High);

    loop {
        // Wait for the button to be pressed
        btn2.wait_for_low().await;
        led2.toggle();
        Timer::after(Duration::from_millis(200)).await;
    }
}
