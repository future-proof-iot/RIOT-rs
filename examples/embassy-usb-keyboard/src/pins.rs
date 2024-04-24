use embassy_nrf::peripherals;

#[cfg(builder = "nrf52840dk")]
riot_rs::define_peripherals!(Buttons {
    btn1: P0_11,
    btn2: P0_12,
    btn3: P0_24,
    btn4: P0_25,
});

#[cfg(builder = "nrf5340dk")]
riot_rs::define_peripherals!(Buttons {
    btn1: P0_23,
    btn2: P0_24,
    btn3: P0_08,
    btn4: P0_09,
});
