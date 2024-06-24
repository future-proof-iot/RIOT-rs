#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use embassy_time::{Duration, Timer};
use riot_rs::embassy::{
    arch::peripherals,
    gpio::{DriveStrength, Input, Output, PinState, Pull},
};

#[cfg(context = "nrf52840dk")]
riot_rs::define_peripherals!(BlinkyPeripherals { led1: P0_13 });

#[cfg(context = "nrf52840dk")]
riot_rs::define_peripherals!(BlinkyButtonPeripherals {
    led2: P0_14,
    btn2: P0_12,
});

#[cfg(context = "nrf5340dk")]
riot_rs::define_peripherals!(BlinkyPeripherals { led1: P0_28 });

#[cfg(context = "nrf5340dk")]
riot_rs::define_peripherals!(BlinkyButtonPeripherals {
    led2: P0_29,
    btn2: P0_24,
});

#[cfg(context = "rp")]
riot_rs::define_peripherals!(BlinkyPeripherals {
    led1: PIN_1,
});

#[riot_rs::task(autostart, peripherals)]
async fn blinky(peripherals: BlinkyPeripherals) {
    // All of the following are possible (not all of them are equivalent):
    //
    // let mut led1 = Output::new(peripherals.led1, PinState::High);
    //
    let mut led1 = Output::builder(peripherals.led1, PinState::High)
        .opt_drive_strength(DriveStrength::default())
        .build();
    //
    // #[cfg(context = "nrf")]
    // let mut led1 = Output::builder(peripherals.led1, PinState::High)
    //     .drive_strength(DriveStrength::Medium)
    //     .build();
    //
    // #[cfg(context = "nrf")]
    // let mut led1 = Output::builder(peripherals.led1, PinState::High)
    //     .drive_strength(DriveStrength::Arch(
    //         riot_rs::embassy::arch::DriveStrength::High,
    //     ))
    //     .build();

    loop {
        led1.toggle();
        Timer::after(Duration::from_millis(500)).await;
    }
}

#[riot_rs::task(autostart, peripherals)]
async fn blinky_button(peripherals: BlinkyButtonPeripherals) {
    let btn2 = Input::new(peripherals.btn2, Pull::Up);
    let mut led2 = Output::new(peripherals.led2, PinState::High);

    loop {
        // If the button is pressed
        if btn2.is_low() {
            led2.toggle();
        }

        Timer::after(Duration::from_millis(200)).await;
    }
}