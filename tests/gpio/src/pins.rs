use riot_rs::embassy::arch::peripherals;

#[cfg(context = "nrf52")]
riot_rs::define_peripherals!(Peripherals {
    pin_0: P0_00,
    pin_1: P0_01,
    pin_2: P0_02,
    pin_3: P0_03,
});

#[cfg(context = "nrf5340")]
riot_rs::define_peripherals!(Peripherals {
    pin_0: P0_00,
    pin_1: P0_01,
    pin_2: P0_04,
    pin_3: P0_05,
});

#[cfg(context = "rp2040")]
riot_rs::define_peripherals!(Peripherals {
    pin_0: PIN_0,
    pin_1: PIN_1,
    pin_2: PIN_2,
    pin_3: PIN_3,
});

#[cfg(context = "esp")]
riot_rs::define_peripherals!(Peripherals {
    pin_0: GPIO_0,
    pin_1: GPIO_1,
    pin_2: GPIO_2,
    pin_3: GPIO_3,
});

#[cfg(context = "stm32")]
riot_rs::define_peripherals!(Peripherals {
    pin_0: PA0,
    pin_1: PA1,
    pin_2: PA2,
    pin_3: PA3,
});
