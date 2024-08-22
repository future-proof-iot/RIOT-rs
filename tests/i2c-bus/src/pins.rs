use riot_rs::embassy::arch::{i2c, peripherals};

#[cfg(context = "esp")]
pub type SensorI2c = i2c::I2C0;
#[cfg(context = "esp")]
riot_rs::define_peripherals!(Peripherals {
    i2c_sda: GPIO_2,
    i2c_scl: GPIO_0,
});

#[cfg(context = "nrf52840")]
pub type SensorI2c = i2c::TWISPI0;
#[cfg(context = "nrf5340")]
pub type SensorI2c = i2c::SERIAL0;
#[cfg(context = "nrf")]
riot_rs::define_peripherals!(Peripherals {
    i2c_sda: P0_00,
    i2c_scl: P0_01,
});

#[cfg(context = "rp")]
pub type SensorI2c = i2c::I2C0;
#[cfg(context = "rp")]
riot_rs::define_peripherals!(Peripherals {
    i2c_sda: PIN_12,
    i2c_scl: PIN_13,
});

#[cfg(context = "stm32h755zitx")]
pub type SensorI2c = i2c::I2C1;
#[cfg(context = "stm32h755zitx")]
riot_rs::define_peripherals!(Peripherals {
    i2c_sda: PB9,
    i2c_scl: PB8,
    i2c_tx_dma: DMA1_CH1,
    i2c_rx_dma: DMA1_CH2,
});

#[cfg(context = "stm32wb55rgvx")]
pub type SensorI2c = i2c::I2C1;
#[cfg(context = "stm32wb55rgvx")]
riot_rs::define_peripherals!(Peripherals {
    i2c_sda: PB9,
    i2c_scl: PB8,
    i2c_tx_dma: DMA1_CH1,
    i2c_rx_dma: DMA1_CH2,
});
