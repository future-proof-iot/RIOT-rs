#![no_main]
#![no_std]
#![feature(impl_trait_in_assoc_type)]
#![feature(used_with_arg)]

mod pins;

#[allow(unused_imports)]
use ariel_os::{
    debug::log::info,
    gpio::{DriveStrength, Input, Level, Output, Pull, Speed},
};

#[ariel_os::task(autostart, peripherals)]
async fn main(peripherals: pins::Peripherals) {
    // Simples constructors
    let _btn_0 = Input::new(peripherals.pin_0, Pull::Up);
    let _led_0 = Output::new(peripherals.pin_1, Level::Low);

    // Builder usage
    let btn_1_builder = Input::builder(peripherals.pin_2, Pull::Up);
    // Set input Schmitt trigger on a HAL that supports configuring it.
    #[cfg(context = "rp2040")]
    let btn_1_builder = btn_1_builder.schmitt_trigger(true);
    let _btn1 = btn_1_builder.build_with_interrupt();

    #[allow(unused_mut)]
    let mut led_1_builder = Output::builder(peripherals.pin_3, Level::Low);
    // Set output drive strength on a HAL that supports configuring it.
    #[cfg(context = "nrf")]
    let led_1_builder = led_1_builder.drive_strength(DriveStrength::High);
    // Set output speed on a HAL that supports configuring it.
    #[cfg(context = "rp2040")]
    let led_1_builder = led_1_builder.speed(Speed::Medium);
    let _led_1 = led_1_builder.build();

    info!("Test passed!");
}
