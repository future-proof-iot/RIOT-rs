use riot_rs::embassy::arch::peripherals;

#[cfg(context = "nrf52840dk")]
riot_rs::define_peripherals!(Peripherals { led1: P0_13 });

#[cfg(context = "nrf52840dk")]
riot_rs::define_peripherals!(ButtonPeripherals {
    led2: P0_14,
    btn2: P0_12,
});

#[cfg(context = "microbit-v2")]
riot_rs::define_peripherals!(Peripherals {
    led_col1: P0_28,
    led1: P0_21,
});

#[cfg(context = "microbit-v2")]
riot_rs::define_peripherals!(ButtonPeripherals {
    btn2: P0_14,
    led2: P0_22
});

#[cfg(context = "nrf5340dk")]
riot_rs::define_peripherals!(Peripherals { led1: P0_28 });

#[cfg(context = "nrf5340dk")]
riot_rs::define_peripherals!(ButtonPeripherals {
    led2: P0_29,
    btn2: P0_24,
});

#[cfg(context = "rp")]
riot_rs::define_peripherals!(Peripherals { led1: PIN_1 });

#[cfg(context = "rp")]
riot_rs::define_peripherals!(ButtonPeripherals {
    led2: PIN_2,
    btn2: PIN_6,
});

#[cfg(context = "esp")]
riot_rs::define_peripherals!(Peripherals { led1: GPIO_0 });

#[cfg(context = "esp")]
riot_rs::define_peripherals!(ButtonPeripherals {
    led2: GPIO_1,
    btn2: GPIO_2,
});

#[cfg(context = "st-nucleo-f401re")]
riot_rs::define_peripherals!(Peripherals { led1: PA0 });

#[cfg(context = "st-nucleo-f401re")]
riot_rs::define_peripherals!(ButtonPeripherals {
    led2: PB0, // nothing connected here
    btn2: PC13,
});

#[cfg(context = "st-nucleo-h755zi-q")]
riot_rs::define_peripherals!(Peripherals { led1: PB0 });

#[cfg(context = "st-nucleo-wb55")]
riot_rs::define_peripherals!(Peripherals { led1: PB5 });

#[cfg(context = "st-nucleo-wb55")]
riot_rs::define_peripherals!(ButtonPeripherals {
    led2: PB0,
    btn2: PD0,
});

#[cfg(all(
    context = "stm32",
    not(any(context = "st-nucleo-wb55", context = "st-nucleo-f401re"))
))]
riot_rs::define_peripherals!(ButtonPeripherals {
    led2: PA7,
    btn2: PA9,
});
