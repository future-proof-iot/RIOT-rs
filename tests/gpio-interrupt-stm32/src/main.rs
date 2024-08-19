#![no_main]
#![no_std]
#![feature(impl_trait_in_assoc_type)]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use riot_rs::{
    debug::log::info,
    embassy::{
        arch::peripherals,
        gpio::{self, Input, Pull},
    },
};

// These pins should be available on all STM32 chips.
#[cfg(context = "stm32")]
riot_rs::define_peripherals!(ButtonPeripherals {
    btn_a0: PA0,
    btn_a1: PA1,
    btn_b0: PB0,
});

#[riot_rs::task(autostart, peripherals)]
async fn main(peripherals: ButtonPeripherals) {
    let _btn_a0 = Input::builder(peripherals.btn_a0, Pull::Up)
        .build_with_interrupt()
        .unwrap();

    // This interrupt uses a different channel, so it should not fail.
    let _btn_a1 = Input::builder(peripherals.btn_a1, Pull::Up)
        .build_with_interrupt()
        .unwrap();

    // This one should return an error, because PB0 uses the same interrupt channel as PA0, which
    // we already used above.
    assert!(matches!(
        Input::builder(peripherals.btn_b0, Pull::Up).build_with_interrupt(),
        // FIXME
        // Err(gpio::input::Error::InterruptChannel(ExtIntRegistry::Error::IntChannelAlreadyUsed)),
        Err(gpio::input::Error::InterruptChannel(_)),
    ));

    info!("Test passed!");
}
