use riot_rs::embassy::arch::{peripherals, spi};

#[cfg(context = "esp")]
pub type SensorSpi = spi::SPI2;
#[cfg(context = "esp")]
riot_rs::define_peripherals!(Peripherals {
    spi_sck: GPIO_0,
    spi_miso: GPIO_1,
    spi_mosi: GPIO_2,
    spi_cs: GPIO_3,
    dma: DMA,
});

#[cfg(context = "nrf52840")]
pub type SensorSpi = spi::SPI2;
#[cfg(context = "nrf5340")]
pub type SensorSpi = spi::SERIAL2;
#[cfg(context = "nrf")]
riot_rs::define_peripherals!(Peripherals {
    spi_sck: P0_00,
    spi_miso: P0_01,
    spi_mosi: P0_04,
    spi_cs: P0_05,
});

#[cfg(context = "rp")]
pub type SensorSpi = spi::SPI0;
#[cfg(context = "rp")]
riot_rs::define_peripherals!(Peripherals {
    spi_sck: PIN_18,
    spi_miso: PIN_16,
    spi_mosi: PIN_19,
    spi_cs: PIN_17,
    spi_tx_dma: DMA_CH0,
    spi_rx_dma: DMA_CH1,
});

#[cfg(context = "stm32h755zitx")]
pub type SensorSpi = spi::SPI2;
#[cfg(context = "stm32h755zitx")]
riot_rs::define_peripherals!(Peripherals {
    spi_sck: PB10,
    spi_miso: PC2,
    spi_mosi: PC3,
    spi_cs: PB12,
    spi_tx_dma: DMA1_CH1,
    spi_rx_dma: DMA1_CH2,
});

#[cfg(context = "stm32wb55rgvx")]
pub type SensorSpi = spi::SPI2;
#[cfg(context = "stm32wb55rgvx")]
riot_rs::define_peripherals!(Peripherals {
    spi_sck: PA9,
    spi_miso: PC2,
    spi_mosi: PC1,
    spi_cs: PC0,
    spi_tx_dma: DMA1_CH1,
    spi_rx_dma: DMA1_CH2,
});
