#[doc(alias = "master")]
pub mod main;

use riot_rs_embassy_common::spi::{BitOrder, Mode};

fn from_mode(mode: Mode) -> esp_hal::spi::SpiMode {
    match mode {
        Mode::Mode0 => esp_hal::spi::SpiMode::Mode0,
        Mode::Mode1 => esp_hal::spi::SpiMode::Mode1,
        Mode::Mode2 => esp_hal::spi::SpiMode::Mode2,
        Mode::Mode3 => esp_hal::spi::SpiMode::Mode3,
    }
}

fn from_bit_order(bit_order: BitOrder) -> esp_hal::spi::SpiBitOrder {
    match bit_order {
        BitOrder::MsbFirst => esp_hal::spi::SpiBitOrder::MSBFirst,
        BitOrder::LsbFirst => esp_hal::spi::SpiBitOrder::LSBFirst,
    }
}

pub fn init(peripherals: &mut crate::OptionalPeripherals) {
    // Take all SPI peripherals and do nothing with them.
    cfg_if::cfg_if! {
        if #[cfg(context = "esp32c6")] {
            let _ = peripherals.SPI2.take().unwrap();
        } else {
            compile_error!("this ESP32 chip is not supported");
        }
    }
}
