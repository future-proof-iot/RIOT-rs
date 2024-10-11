#[doc(alias = "master")]
pub mod main;

use riot_rs_embassy_common::spi::{BitOrder, Mode};

fn from_mode(mode: Mode) -> embassy_nrf::spim::Mode {
    match mode {
        Mode::Mode0 => embassy_nrf::spim::MODE_0,
        Mode::Mode1 => embassy_nrf::spim::MODE_1,
        Mode::Mode2 => embassy_nrf::spim::MODE_2,
        Mode::Mode3 => embassy_nrf::spim::MODE_3,
    }
}

fn from_bit_order(bit_order: BitOrder) -> embassy_nrf::spim::BitOrder {
    match bit_order {
        BitOrder::MsbFirst => embassy_nrf::spim::BitOrder::MSB_FIRST,
        BitOrder::LsbFirst => embassy_nrf::spim::BitOrder::LSB_FIRST,
    }
}

pub fn init(peripherals: &mut crate::OptionalPeripherals) {
    // Take all SPI peripherals and do nothing with them.
    cfg_if::cfg_if! {
        if #[cfg(context = "nrf52840")] {
            let _ = peripherals.SPI2.take().unwrap();
            let _ = peripherals.SPI3.take().unwrap();
        } else if #[cfg(context = "nrf5340")] {
            let _ = peripherals.SERIAL2.take().unwrap();
            let _ = peripherals.SERIAL3.take().unwrap();
        } else {
            compile_error!("this nRF chip is not supported");
        }
    }
}
