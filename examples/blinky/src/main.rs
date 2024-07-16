#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

mod pins;

use embassy_time::{Duration, Timer};
use riot_rs::embassy::gpio::{Level, Output};

#[riot_rs::task(autostart, peripherals)]
async fn blinky(peripherals: pins::LedPeripherals) {
    let mut led = Output::new(peripherals.led, Level::Low);

    loop {
        led.toggle();
        Timer::after(Duration::from_millis(500)).await;
    }
}
