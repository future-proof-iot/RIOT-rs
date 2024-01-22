use embassy_nrf::peripherals;
use riot_rs::define_peripherals;

#[cfg(builder = "nrf52840dk")]
define_peripherals!(Buttons {
    btn1: P0_11,
    btn2: P0_12,
    btn3: P0_24,
    btn4: P0_25,
});
