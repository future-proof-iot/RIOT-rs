//! This example is merely to illustrate and test raw bus usage.
//!
//! Please use [`riot_rs::sensors`] instead for a high-level sensor abstraction that is
//! architecture-agnostic.
//!
//! This example requires a LIS3DH sensor (3-axis accelerometer).
#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use embassy_sync::mutex::Mutex;
use embedded_hal_async::spi::{Operation, SpiDevice as _};
use riot_rs::{
    debug::{exit, log::info, EXIT_SUCCESS},
    embassy::{
        arch::{peripherals, spi},
        gpio,
        spi::SpiDevice,
    },
};

// WHO_AM_I register of the LIS3DH sensor
const WHO_AM_I_REG_ADDR: u8 = 0x0f;

// FIXME
#[cfg(context = "esp")]
riot_rs::define_peripherals!(Peripherals {
    spi_peripheral: SPI2,
    spi_sck: GPIO_0,
    spi_miso: GPIO_1,
    spi_mosi: GPIO_2,
    spi_cs: GPIO_3,
    dma: DMA,
});

#[cfg(context = "rp")]
riot_rs::define_peripherals!(Peripherals {
    spi_peripheral: SPI0,
    spi_sck: PIN_18,
    spi_miso: PIN_16,
    spi_mosi: PIN_19,
    spi_cs: PIN_17,
    spi_tx_dma: DMA_CH0,
    spi_rx_dma: DMA_CH1,
});

#[cfg(context = "stm32h755zitx")]
riot_rs::define_peripherals!(Peripherals {
    spi_peripheral: SPI2,
    spi_sck: PB10,
    spi_miso: PC2,
    spi_mosi: PC3,
    spi_cs: PB12,
    spi_tx_dma: DMA1_CH1,
    spi_rx_dma: DMA1_CH2,
});

#[cfg(context = "stm32wb55rgvx")]
riot_rs::define_peripherals!(Peripherals {
    spi_peripheral: SPI2,
    spi_sck: PA9,
    spi_miso: PC2,
    spi_mosi: PC1,
    spi_cs: PC0,
    spi_tx_dma: DMA1_CH1,
    spi_rx_dma: DMA1_CH2,
});

pub static SPI_BUS: once_cell::sync::OnceCell<
    Mutex<embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex, spi::Spi>,
> = once_cell::sync::OnceCell::new();

#[riot_rs::task(autostart, peripherals)]
async fn main(peripherals: Peripherals) {
    let mut spi_config = spi::Config::default();
    spi_config.frequency = spi::Frequency::M1;
    spi_config.mode = spi::Mode::Mode3;

    let dma = esp_hal::dma::Dma::new(peripherals.dma);

    #[cfg(context = "esp")]
    let spi_bus = spi::Spi::SPI2(spi::SpiSPI2::new(
        peripherals.spi_peripheral,
        peripherals.spi_sck,
        peripherals.spi_miso,
        peripherals.spi_mosi,
        dma.channel1,
        spi_config,
    ));

    #[cfg(context = "rp")]
    let spi_bus = spi::Spi::SPI0(spi::SpiSPI0::new(
        peripherals.spi_peripheral,
        peripherals.spi_sck,
        peripherals.spi_miso,
        peripherals.spi_mosi,
        peripherals.spi_tx_dma,
        peripherals.spi_rx_dma,
        spi_config,
    ));

    #[cfg(context = "stm32h755zitx")]
    let spi_bus = spi::Spi::SPI2(spi::SpiSPI2::new(
        peripherals.spi_peripheral,
        peripherals.spi_sck,
        peripherals.spi_miso,
        peripherals.spi_mosi,
        peripherals.spi_tx_dma,
        peripherals.spi_rx_dma,
        spi_config,
    ));

    #[cfg(context = "stm32wb55rgvx")]
    let spi_bus = spi::Spi::SPI2(spi::SpiSPI2::new(
        peripherals.spi_peripheral,
        peripherals.spi_sck,
        peripherals.spi_miso,
        peripherals.spi_mosi,
        peripherals.spi_tx_dma,
        peripherals.spi_rx_dma,
        spi_config,
    ));

    let _ = SPI_BUS.set(Mutex::new(spi_bus));

    let cs_output = gpio::Output::new(peripherals.spi_cs, gpio::Level::High);

    let mut spi_device = SpiDevice::new(SPI_BUS.get().unwrap(), cs_output);

    let mut id = [0];
    spi_device
        .transaction(&mut [
            Operation::Write(&[get_spi_read_command(WHO_AM_I_REG_ADDR)]),
            Operation::TransferInPlace(&mut id),
        ])
        .await
        .unwrap();

    let who_am_i = id[0];
    info!("LIS3DH WHO_AM_I_COMMAND register value: 0x{:x}", who_am_i);
    assert_eq!(who_am_i, 0x33);

    info!("Test passed!");

    exit(EXIT_SUCCESS);
}

fn get_spi_read_command(addr: u8) -> u8 {
    addr | 0x80
}
