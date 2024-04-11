#[allow(clippy::unused_imports)]
use riot_rs::embassy::arch::peripherals;

#[cfg(all(feature = "button-readings", builder = "nrf52840dk"))]
riot_rs::define_peripherals!(Buttons {
    btn1: P0_11,
    btn2: P0_12,
    btn3: P0_24,
    btn4: P0_25,
});

#[cfg(context = "nrf52840")]
riot_rs::define_peripherals!(Temp { temp: TEMP });

riot_rs::group_peripherals!(Peripherals {
    #[cfg(feature = "button-readings")]
    buttons: Buttons,
    #[cfg(context = "nrf52840")]
    temp: Temp,
});
