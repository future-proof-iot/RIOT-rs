#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use embassy_time::{Duration, Timer};
use riot_rs::embassy::{arch::peripherals, gpio};

#[cfg(context = "nrf52840dk")]
riot_rs::define_peripherals!(Peripherals {
    led1: P0_13,
    led2: P0_14,
    led3: P0_15,
    led4: P0_16,
});

#[cfg(context = "nrf5340dk")]
riot_rs::define_peripherals!(BlinkyPeripherals { led1: P0_28 });

#[cfg(context = "nrf5340dk")]
riot_rs::define_peripherals!(BlinkyButtonPeripherals {
    led2: P0_29,
    btn2: P0_24,
});

#[cfg(context = "rp")]
riot_rs::define_peripherals!(Peripherals {
    led1: PIN_1,
    led2: PIN_2,
    led3: PIN_3,
    led4: PIN_4,
});

#[riot_rs::task(autostart, peripherals)]
async fn main(peripherals: Peripherals) {
    // All of the following are possible (not all of them are equivalent):
    //
    // let mut led1 = gpio::Output::new(peripherals.led1, gpio::PinState::High);
    //
    let mut led1 = gpio::Output::builder(peripherals.led1, gpio::PinState::High)
        .opt_drive_strength(gpio::DriveStrength::default())
        .build();
    //
    // #[cfg(context = "nrf")]
    // let mut led1 = gpio::Output::builder(peripherals.led1, gpio::PinState::High)
    //     .drive_strength(gpio::DriveStrength::Medium)
    //     .build();
    //
    // #[cfg(context = "nrf")]
    // let mut led1 = gpio::Output::builder(peripherals.led1, gpio::PinState::High)
    //     .drive_strength(gpio::DriveStrength::Arch(
    //         riot_rs::embassy::arch::gpio::DriveStrength::High,
    //     ))
    //     .build();

    loop {
        led1.toggle();
        Timer::after(Duration::from_millis(500)).await;
    }
}
