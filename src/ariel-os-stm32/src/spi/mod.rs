//! Provides support for the SPI communication bus.

#[doc(alias = "master")]
pub mod main;

use ariel_os_embassy_common::spi::{BitOrder, Mode};

fn from_mode(mode: Mode) -> embassy_stm32::spi::Mode {
    match mode {
        Mode::Mode0 => embassy_stm32::spi::MODE_0,
        Mode::Mode1 => embassy_stm32::spi::MODE_1,
        Mode::Mode2 => embassy_stm32::spi::MODE_2,
        Mode::Mode3 => embassy_stm32::spi::MODE_3,
    }
}

fn from_bit_order(bit_order: BitOrder) -> embassy_stm32::spi::BitOrder {
    match bit_order {
        BitOrder::MsbFirst => embassy_stm32::spi::BitOrder::MsbFirst,
        BitOrder::LsbFirst => embassy_stm32::spi::BitOrder::LsbFirst,
    }
}

#[doc(hidden)]
pub fn init(peripherals: &mut crate::OptionalPeripherals) {
    // This macro has to be defined in this function so that the `peripherals` variables exists.
    macro_rules! take_all_spi_peripherals {
        ($peripherals:ident, $( $peripheral:ident ),*) => {
            $(
                let _ = peripherals.$peripheral.take().unwrap();
            )*
        }
    }

    // Take all SPI peripherals and do nothing with them.
    cfg_if::cfg_if! {
        if #[cfg(context = "stm32f401retx")] {
            take_all_spi_peripherals!(Peripherals, SPI1, SPI2, SPI3);
        } else if #[cfg(context = "stm32h755zitx")] {
            take_all_spi_peripherals!(Peripherals, SPI1, SPI2, SPI3, SPI4, SPI5, SPI6);
        } else if #[cfg(context = "stm32wb55rgvx")] {
            take_all_spi_peripherals!(Peripherals, SPI1, SPI2);
        } else {
            compile_error!("this STM32 chip is not supported");
        }
    }
}
