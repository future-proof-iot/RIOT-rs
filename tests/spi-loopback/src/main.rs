//! This example is merely to illustrate and test raw bus usage.
//!
//! Please use [`ariel_os::sensors`] instead for a high-level sensor abstraction that is
//! HAL-agnostic.
#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]
#![feature(impl_trait_in_assoc_type)]

mod pins;

use ariel_os::{
    debug::{
        exit,
        log::{debug, info},
        EXIT_SUCCESS,
    },
    gpio, hal,
    spi::{
        main::{highest_freq_in, Kilohertz, SpiDevice},
        Mode,
    },
};
use embassy_sync::mutex::Mutex;
use embedded_hal_async::spi::SpiDevice as _;

pub static SPI_BUS: once_cell::sync::OnceCell<
    Mutex<embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex, hal::spi::main::Spi>,
> = once_cell::sync::OnceCell::new();

#[ariel_os::task(autostart, peripherals)]
async fn main(peripherals: pins::Peripherals) {
    let mut spi_config = hal::spi::main::Config::default();
    spi_config.frequency = const { highest_freq_in(Kilohertz::kHz(1000)..=Kilohertz::kHz(2000)) };
    debug!("Selected frequency: {}", spi_config.frequency);
    spi_config.mode = if !cfg!(context = "esp") {
        Mode::Mode3
    } else {
        // FIXME: the sensor datasheet does say SPI mode 3, not mode 0
        Mode::Mode0
    };

    let spi_bus = pins::SensorSpi::new(
        peripherals.spi_sck,
        peripherals.spi_miso,
        peripherals.spi_mosi,
        spi_config,
    );

    let _ = SPI_BUS.set(Mutex::new(spi_bus));

    let cs_output = gpio::Output::new(peripherals.spi_cs, gpio::Level::High);
    let mut spi_device = SpiDevice::new(SPI_BUS.get().unwrap(), cs_output);

    let out = [0u8, 1, 2, 3, 4, 5, 6, 7];
    let mut in_ = [0u8; 8];
    spi_device.transfer(&mut in_, &out).await.unwrap();

    info!("got 0x{:x}", &in_);

    assert_eq!(out, in_);

    info!("Test passed!");

    exit(EXIT_SUCCESS);
}
