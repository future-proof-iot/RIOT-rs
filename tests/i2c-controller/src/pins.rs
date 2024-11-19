use riot_rs::hal::{i2c, peripherals};

#[cfg(context = "esp")]
pub type SensorI2c = i2c::controller::I2C0;
#[cfg(context = "esp")]
riot_rs::define_peripherals!(Peripherals {
    i2c_sda: GPIO_2,
    i2c_scl: GPIO_0,
});

#[cfg(any(context = "nrf52833", context = "nrf52840"))]
pub type SensorI2c = i2c::controller::TWISPI0;
#[cfg(context = "nrf5340")]
pub type SensorI2c = i2c::controller::SERIAL0;
#[cfg(all(context = "nrf", not(context = "microbit-v2")))]
riot_rs::define_peripherals!(Peripherals {
    i2c_sda: P0_00,
    i2c_scl: P0_01,
});
#[cfg(context = "microbit-v2")]
riot_rs::define_peripherals!(Peripherals {
    i2c_sda: P0_16,
    i2c_scl: P0_08,
});

#[cfg(context = "rp")]
pub type SensorI2c = i2c::controller::I2C0;
#[cfg(context = "rp")]
riot_rs::define_peripherals!(Peripherals {
    i2c_sda: PIN_12,
    i2c_scl: PIN_13,
});

#[cfg(context = "stm32h755zitx")]
pub type SensorI2c = i2c::controller::I2C1;
#[cfg(context = "stm32h755zitx")]
riot_rs::define_peripherals!(Peripherals {
    i2c_sda: PB9,
    i2c_scl: PB8,
});

#[cfg(context = "stm32wb55rgvx")]
pub type SensorI2c = i2c::controller::I2C1;
#[cfg(context = "stm32wb55rgvx")]
riot_rs::define_peripherals!(Peripherals {
    i2c_sda: PB9,
    i2c_scl: PB8,
});
