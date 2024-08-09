use embassy_nrf::{
    bind_interrupts,
    gpio::{self, Pin as GpioPin},
    peripherals,
    spim::{InterruptHandler, Spim},
    Peripheral,
};

use crate::spi::impl_async_spibus_for_driver_enum;

pub use embassy_nrf::spim::Frequency;

#[derive(Clone)]
#[non_exhaustive]
pub struct Config {
    pub frequency: Frequency,
    pub mode: Mode,
    pub bit_order: BitOrder,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frequency: Frequency::M1,
            mode: Mode::Mode0,
            bit_order: BitOrder::MsbFirst,
        }
    }
}

#[derive(Copy, Clone)]
pub enum Mode {
    Mode0,
    Mode1,
    Mode2,
    Mode3,
}

// https://en.wikipedia.org/wiki/Serial_Peripheral_Interface#Mode_numbers
impl From<Mode> for embassy_nrf::spim::Mode {
    fn from(mode: Mode) -> Self {
        match mode {
            Mode::Mode0 => embassy_nrf::spim::MODE_0,
            Mode::Mode1 => embassy_nrf::spim::MODE_1,
            Mode::Mode2 => embassy_nrf::spim::MODE_2,
            Mode::Mode3 => embassy_nrf::spim::MODE_3,
        }
    }
}

#[derive(Copy, Clone)]
pub enum BitOrder {
    MsbFirst,
    LsbFirst,
}

impl From<BitOrder> for embassy_nrf::spim::BitOrder {
    fn from(bit_order: BitOrder) -> Self {
        match bit_order {
            BitOrder::MsbFirst => embassy_nrf::spim::BitOrder::MSB_FIRST,
            BitOrder::LsbFirst => embassy_nrf::spim::BitOrder::LSB_FIRST,
        }
    }
}

macro_rules! define_spi_drivers {
    ($( $interrupt:ident => $peripheral:ident ),* $(,)?) => {
        // paste allows to create new identifiers by concatenation using `[<foo bar>]`.
        paste::paste! {
            $(
                /// Peripheral-specific SPI driver.
                pub struct [<Spi $peripheral>] {
                    spim: Spim<'static, peripherals::$peripheral>,
                }

                impl [<Spi $peripheral>] {
                    #[must_use]
                    pub fn new(
                        spim_peripheral: impl Peripheral<P = peripherals::$peripheral> + 'static,
                        sck_pin: impl Peripheral<P: GpioPin> + 'static,
                        miso_pin: impl Peripheral<P: GpioPin> + 'static,
                        mosi_pin: impl Peripheral<P: GpioPin> + 'static,
                        config: Config,
                    ) -> Self {
                        let mut spi_config = embassy_nrf::spim::Config::default();
                        spi_config.frequency = config.frequency;
                        spi_config.mode = config.mode.into();
                        spi_config.bit_order = config.bit_order.into();

                        bind_interrupts!(
                            struct Irqs {
                                $interrupt => InterruptHandler<peripherals::$peripheral>;
                            }
                        );

                        let spim = Spim::new(
                            spim_peripheral,
                            Irqs,
                            sck_pin,
                            miso_pin,
                            mosi_pin,
                            spi_config,
                        );

                        Self { spim }
                    }
                }
            )*

            /// Peripheral-agnostic driver.
            pub enum Spi {
                $( $peripheral([<Spi $peripheral>]), )*
            }

            impl embedded_hal_async::spi::ErrorType for Spi {
                type Error = embassy_nrf::spim::Error;
            }

            impl_async_spibus_for_driver_enum!(Spi, $( $peripheral ),*);
        }
    };
}

// FIXME: support other nRF archs
// Define a driver per peripheral
#[cfg(context = "nrf52840")]
define_spi_drivers!(
    // FIXME
    // SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0 => TWISPI0,
    // SPIM1_SPIS1_TWIM1_TWIS1_SPI1_TWI1 => TWISPI1,
    SPIM2_SPIS2_SPI2 => SPI2,
    SPIM3 => SPI3,
);
#[cfg(context = "nrf5340")]
define_spi_drivers!(
    SERIAL2 => SERIAL2,
    SERIAL3 => SERIAL3,
);
