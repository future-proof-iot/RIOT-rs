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
#![feature(impl_trait_in_assoc_type)]

mod pins;

use embassy_sync::mutex::Mutex;
use embedded_hal_async::spi::{Operation, SpiDevice as _};
use riot_rs::{
    debug::{exit, log::info, EXIT_SUCCESS},
    embassy::{
        arch::spi,
        gpio,
        spi::{Frequency, Mode, SpiDevice},
    },
};

// WHO_AM_I register of the LIS3DH sensor
const WHO_AM_I_REG_ADDR: u8 = 0x0f;

// pub static SPI_BUS: once_cell::sync::OnceCell<
//     Mutex<embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex, spi::Spi>,
// > = once_cell::sync::OnceCell::new();

#[riot_rs::task(autostart, peripherals)]
async fn main(peripherals: pins::Peripherals) {
    let mut spi_config = spi::Config::default();
    spi_config.frequency = Frequency::_125k.into();
    spi_config.mode = if !cfg!(context = "esp") {
        Mode::Mode3
    } else {
        // FIXME: the sensor datasheet does match with SPI mode 3, not mode 0
        Mode::Mode0
    };

    // FIXME
    #[cfg(context = "esp")]
    let dma = esp_hal::dma::Dma::new(peripherals.dma);

    let spi_bus = pins::SensorSpi::new(
        peripherals.spi_sck,
        peripherals.spi_miso,
        peripherals.spi_mosi,
        #[cfg(context = "esp")]
        dma.channel1,
        #[cfg(any(context = "rp", context = "stm32"))]
        peripherals.spi_tx_dma,
        #[cfg(any(context = "rp", context = "stm32"))]
        peripherals.spi_rx_dma,
        spi_config,
    );

    let spi_bus = static_cell::make_static!(Mutex::new(spi_bus));
    // let _ = SPI_BUS.set(Mutex::new(spi_bus));

    let cs_output = gpio::Output::new(peripherals.spi_cs, gpio::Level::High);

    let mut spi_device = SpiDevice::new(&*spi_bus, cs_output);

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
