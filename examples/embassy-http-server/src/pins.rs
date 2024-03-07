use riot_rs::define_peripherals;

#[cfg(feature = "button-readings")]
use riot_rs::embassy::arch::peripherals;

#[cfg(all(feature = "button-readings", builder = "nrf52840dk"))]
define_peripherals!(Buttons {
    btn1: P0_11,
    btn2: P0_12,
    btn3: P0_24,
    btn4: P0_25,
});

#[cfg(context = "nrf52840")]
define_peripherals!(Temp { temp: TEMP });
