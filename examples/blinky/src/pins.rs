use riot_rs::embassy::arch::peripherals;

#[cfg(context = "nrf52840dk")]
riot_rs::define_peripherals!(LedPeripherals { led: P0_13 });

#[cfg(context = "nrf5340dk")]
riot_rs::define_peripherals!(LedPeripherals { led: P0_28 });

#[cfg(context = "rp")]
riot_rs::define_peripherals!(LedPeripherals { led: PIN_1 });

#[cfg(context = "esp")]
riot_rs::define_peripherals!(LedPeripherals { led: GPIO_0 });

#[cfg(context = "stm32")]
riot_rs::define_peripherals!(LedPeripherals { led: PA6 });
