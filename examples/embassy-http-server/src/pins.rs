use riot_rs::define_peripherals;

#[cfg(all(feature = "button-readings", builder = "nrf52840dk"))]
use embassy_nrf::peripherals;

#[cfg(all(feature = "button-readings", builder = "nrf52840dk"))]
define_peripherals!(Buttons {
    btn1: P0_11,
    btn2: P0_12,
    btn3: P0_24,
    btn4: P0_25,
});
