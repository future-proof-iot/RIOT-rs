#![no_main]
#![no_std]
#![feature(impl_trait_in_assoc_type)]
#![feature(used_with_arg)]

use ariel_os::{
    debug::log::info,
    gpio::{self, Input, Pull},
    hal::peripherals,
};

#[cfg(context = "nrf51")]
todo!();

#[cfg(context = "nrf52")]
ariel_os::define_peripherals!(ButtonPeripherals {
    btn_0: P0_00,
    btn_1: P0_01,
    btn_2: P0_02,
    btn_3: P0_03,
    btn_4: P0_04,
    btn_5: P0_05,
    btn_6: P0_06,
    btn_7: P0_07,
    btn_8: P0_08,
});

#[cfg(context = "nrf5340")]
ariel_os::define_peripherals!(ButtonPeripherals {
    btn_0: P0_00,
    btn_1: P0_01,
    btn_2: P0_04,
    btn_3: P0_05,
    btn_4: P0_06,
    btn_5: P0_07,
    btn_6: P0_08,
    btn_7: P0_09,
    btn_8: P0_10,
});

#[ariel_os::task(autostart, peripherals)]
async fn main(peripherals: ButtonPeripherals) {
    let _btn_0 = Input::builder(peripherals.btn_0, Pull::Up)
        .build_with_interrupt()
        .unwrap();
    let _btn_1 = Input::builder(peripherals.btn_1, Pull::Up)
        .build_with_interrupt()
        .unwrap();
    let _btn_2 = Input::builder(peripherals.btn_2, Pull::Up)
        .build_with_interrupt()
        .unwrap();
    let _btn_3 = Input::builder(peripherals.btn_3, Pull::Up)
        .build_with_interrupt()
        .unwrap();

    // This one should return an error, because nRF51s have only 4 interrupt channels.
    #[cfg(context = "nrf51")]
    {
        info!("Testing on `nrf51`");
        assert!(matches!(
            Input::builder(peripherals.btn_8, Pull::Up).build_with_interrupt(),
            // FIXME
            // Err(gpio::input::Error::InterruptChannel(ExtIntRegistry::Error::NoIntChannelAvailable)),
            Err(gpio::input::Error::InterruptChannel(_)),
        ));
    }

    #[cfg(not(context = "nrf51"))]
    {
        info!("Testing on `nrf` other than `nrf51`");
        let _btn_4 = Input::builder(peripherals.btn_4, Pull::Up)
            .build_with_interrupt()
            .unwrap();
        let _btn_5 = Input::builder(peripherals.btn_5, Pull::Up)
            .build_with_interrupt()
            .unwrap();
        let _btn_6 = Input::builder(peripherals.btn_6, Pull::Up)
            .build_with_interrupt()
            .unwrap();
        let _btn_7 = Input::builder(peripherals.btn_7, Pull::Up)
            .build_with_interrupt()
            .unwrap();

        // This one should return an error, because other nRFs have only 8 interrupt channels.
        assert!(matches!(
            Input::builder(peripherals.btn_8, Pull::Up).build_with_interrupt(),
            // FIXME
            // Err(gpio::input::Error::InterruptChannel(ExtIntRegistry::Error::NoIntChannelAvailable)),
            Err(gpio::input::Error::InterruptChannel(_)),
        ));
    }

    info!("Test passed!");
}
