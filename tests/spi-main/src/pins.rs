use riot_rs::arch::{peripherals, spi};

#[cfg(context = "esp")]
pub type SensorSpi = spi::main::SPI2;
#[cfg(context = "esp")]
riot_rs::define_peripherals!(Peripherals {
    spi_sck: GPIO_0,
    spi_miso: GPIO_1,
    spi_mosi: GPIO_2,
    spi_cs: GPIO_3,
});

#[cfg(context = "nrf52840")]
pub type SensorSpi = spi::main::SPI3;
#[cfg(context = "nrf52840")]
riot_rs::define_peripherals!(Peripherals {
    spi_sck: P0_28,
    spi_miso: P0_30,
    spi_mosi: P0_29,
    spi_cs: P0_31,
});

#[cfg(context = "nrf5340")]
pub type SensorSpi = spi::main::SERIAL2;
#[cfg(context = "nrf5340")]
riot_rs::define_peripherals!(Peripherals {
    spi_sck: P0_06,
    spi_miso: P0_25,
    spi_mosi: P0_07,
    spi_cs: P0_26,
});

#[cfg(context = "rp")]
pub type SensorSpi = spi::main::SPI0;
#[cfg(context = "rp")]
riot_rs::define_peripherals!(Peripherals {
    spi_sck: PIN_18,
    spi_miso: PIN_16,
    spi_mosi: PIN_19,
    spi_cs: PIN_17,
});

#[cfg(context = "stm32h755zitx")]
pub type SensorSpi = spi::main::SPI2;
#[cfg(context = "stm32h755zitx")]
riot_rs::define_peripherals!(Peripherals {
    spi_sck: PB10,
    spi_miso: PC2,
    spi_mosi: PC3,
    spi_cs: PB12,
});

#[cfg(context = "stm32wb55rgvx")]
pub type SensorSpi = spi::main::SPI2;
#[cfg(context = "stm32wb55rgvx")]
riot_rs::define_peripherals!(Peripherals {
    spi_sck: PA9,
    spi_miso: PC2,
    spi_mosi: PC1,
    spi_cs: PC0,
});
