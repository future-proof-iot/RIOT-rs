use ariel_os::hal::peripherals;

#[cfg(context = "nrf52")]
ariel_os::hal::define_peripherals!(Peripherals {
    pin_0: P0_00,
    pin_1: P0_01,
    pin_2: P0_02,
    pin_3: P0_03,
});

#[cfg(context = "nrf5340")]
ariel_os::hal::define_peripherals!(Peripherals {
    pin_0: P0_00,
    pin_1: P0_01,
    pin_2: P0_04,
    pin_3: P0_05,
});

#[cfg(context = "rp2040")]
ariel_os::hal::define_peripherals!(Peripherals {
    pin_0: PIN_0,
    pin_1: PIN_1,
    pin_2: PIN_2,
    pin_3: PIN_3,
});

#[cfg(context = "esp32")]
ariel_os::hal::define_peripherals!(Peripherals {
    pin_0: GPIO_16,
    pin_1: GPIO_17,
    pin_2: GPIO_18,
    pin_3: GPIO_19,
});

#[cfg(all(context = "esp", not(context = "esp32")))]
ariel_os::hal::define_peripherals!(Peripherals {
    pin_0: GPIO_0,
    pin_1: GPIO_1,
    pin_2: GPIO_2,
    pin_3: GPIO_3,
});

#[cfg(context = "stm32")]
ariel_os::hal::define_peripherals!(Peripherals {
    pin_0: PA0,
    pin_1: PA1,
    pin_2: PA2,
    pin_3: PA3,
});
