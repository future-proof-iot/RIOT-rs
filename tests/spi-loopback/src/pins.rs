use ariel_os::hal::{peripherals, spi};

#[cfg(context = "esp")]
pub type SensorSpi = spi::main::SPI2;
#[cfg(context = "esp")]
ariel_os::define_peripherals!(Peripherals {
    spi_sck: GPIO0,
    spi_miso: GPIO1,
    spi_mosi: GPIO2,
    spi_cs: GPIO3,
});

#[cfg(context = "nrf52833")]
pub type SensorSpi = spi::main::SPI3;
#[cfg(context = "nrf52833")]
ariel_os::define_peripherals!(Peripherals {
    spi_sck: P0_17,  // SPI_EXT_SCK on the microbit-v2
    spi_miso: P0_01, // SPI_EXT_MISO on the microbit-v2
    spi_mosi: P0_13, // SPI_EXT_MOSI on the microbit-v2
    spi_cs: P0_04,   // RING2 on the microbit-v2
});

// Side SPI of Arduino v3 connector
#[cfg(context = "nrf52840")]
pub type SensorSpi = spi::main::SPI3;
#[cfg(context = "nrf52840")]
ariel_os::define_peripherals!(Peripherals {
    spi_sck: P1_15,
    spi_miso: P1_14,
    spi_mosi: P1_13,
    spi_cs: P1_12,
});

// Side SPI of Arduino v3 connector
#[cfg(context = "nrf5340")]
pub type SensorSpi = spi::main::SERIAL2;
#[cfg(context = "nrf5340")]
ariel_os::define_peripherals!(Peripherals {
    spi_sck: P1_15,
    spi_miso: P1_14,
    spi_mosi: P1_13,
    spi_cs: P1_12,
});

#[cfg(context = "rp")]
pub type SensorSpi = spi::main::SPI0;
#[cfg(context = "rp")]
ariel_os::define_peripherals!(Peripherals {
    spi_sck: PIN_18,
    spi_miso: PIN_16,
    spi_mosi: PIN_19,
    spi_cs: PIN_17,
});

// Side SPI of Arduino v3 connector
#[cfg(context = "stm32h755zitx")]
pub type SensorSpi = spi::main::SPI1;
#[cfg(context = "stm32h755zitx")]
ariel_os::define_peripherals!(Peripherals {
    spi_sck: PA5,
    spi_miso: PA6,
    spi_mosi: PB5,
    spi_cs: PD14,
});

// Side SPI of Arduino v3 connector
#[cfg(context = "stm32wb55rgvx")]
pub type SensorSpi = spi::main::SPI1;
#[cfg(context = "stm32wb55rgvx")]
ariel_os::define_peripherals!(Peripherals {
    spi_sck: PA5,
    spi_miso: PA6,
    spi_mosi: PA7,
    spi_cs: PA4,
});

// Side SPI of Arduino v3 connector
#[cfg(context = "stm32f401retx")]
pub type SensorSpi = spi::main::SPI1;
#[cfg(context = "stm32f401retx")]
ariel_os::define_peripherals!(Peripherals {
    spi_sck: PA5,
    spi_miso: PA6,
    spi_mosi: PA7,
    spi_cs: PA4,
});
