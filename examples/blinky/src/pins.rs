use riot_rs::arch::peripherals;

#[cfg(context = "microbit-v2")]
riot_rs::define_peripherals!(LedPeripherals {
    led_col1: P0_28,
    led: P0_21,
});

#[cfg(context = "nrf52840dk")]
riot_rs::define_peripherals!(LedPeripherals { led: P0_13 });

#[cfg(context = "nrf5340dk")]
riot_rs::define_peripherals!(LedPeripherals { led: P0_28 });

#[cfg(context = "rp2040")]
riot_rs::define_peripherals!(LedPeripherals { led: PIN_1 });

#[cfg(context = "rp2350")]
riot_rs::define_peripherals!(LedPeripherals { led: PIN_25 });

#[cfg(context = "esp")]
riot_rs::define_peripherals!(LedPeripherals { led: GPIO_0 });

#[cfg(context = "st-nucleo-f401re")]
riot_rs::define_peripherals!(LedPeripherals { led: PA5 });

#[cfg(context = "st-nucleo-h755zi-q")]
riot_rs::define_peripherals!(LedPeripherals { led: PB0 });

#[cfg(context = "st-nucleo-wb55")]
riot_rs::define_peripherals!(LedPeripherals { led: PB5 });
