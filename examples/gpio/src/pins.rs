use riot_rs::hal::peripherals;

#[cfg(context = "nrf52840dk")]
riot_rs::define_peripherals!(Peripherals {
    led1: P0_13,
    btn1: P0_11
});

#[cfg(context = "microbit-v2")]
riot_rs::define_peripherals!(Peripherals {
    led_col1: P0_28,
    led1: P0_21,
    btn1: P0_14
});

#[cfg(context = "nrf5340dk")]
riot_rs::define_peripherals!(Peripherals {
    led1: P0_28,
    btn1: P0_23
});

#[cfg(context = "rp")]
riot_rs::define_peripherals!(Peripherals {
    led1: PIN_1,
    btn1: PIN_2
});

#[cfg(context = "esp")]
riot_rs::define_peripherals!(Peripherals {
    led1: GPIO_0,
    btn1: GPIO_1
});

#[cfg(context = "st-nucleo-f401re")]
riot_rs::define_peripherals!(Peripherals {
    led1: PA5,
    btn1: PC13
});

#[cfg(context = "st-nucleo-h755zi-q")]
riot_rs::define_peripherals!(Peripherals {
    btn1: PC13,
    led1: PB0
});

#[cfg(context = "st-nucleo-wb55")]
riot_rs::define_peripherals!(Peripherals {
    led1: PB5,
    btn1: PC4
});
