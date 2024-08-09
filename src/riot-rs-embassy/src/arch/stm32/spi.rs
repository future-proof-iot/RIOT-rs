use embassy_stm32::{
    gpio,
    mode::Async,
    peripherals,
    spi::{MisoPin, MosiPin, RxDma, SckPin, Spi as InnerSpi, TxDma},
    time::Hertz,
    Peripheral,
};

use crate::spi::impl_async_spibus_for_driver_enum;

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

// Possible values are copied from embassy-nrf
// TODO: check how well this matches the STM32 capabilities
#[derive(Copy, Clone)]
#[repr(u32)]
pub enum Frequency {
    K125 = 125_000,
    K250 = 250_000,
    K500 = 500_00,
    M1 = 1_000_000,
    M2 = 2_000_000,
    M4 = 4_000_000,
    M8 = 8_000_000,
    M16 = 16_000_000,
    M32 = 32_000_000,
}

impl From<Frequency> for Hertz {
    fn from(freq: Frequency) -> Self {
        match freq {
            Frequency::K125 => Hertz::khz(125),
            Frequency::K250 => Hertz::khz(250),
            Frequency::K500 => Hertz::khz(500),
            Frequency::M1 => Hertz::mhz(1),
            Frequency::M2 => Hertz::mhz(2),
            Frequency::M4 => Hertz::mhz(4),
            Frequency::M8 => Hertz::mhz(8),
            Frequency::M16 => Hertz::mhz(16),
            Frequency::M32 => Hertz::mhz(32),
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
impl From<Mode> for embassy_stm32::spi::Mode {
    fn from(mode: Mode) -> Self {
        match mode {
            Mode::Mode0 => embassy_stm32::spi::MODE_0,
            Mode::Mode1 => embassy_stm32::spi::MODE_1,
            Mode::Mode2 => embassy_stm32::spi::MODE_2,
            Mode::Mode3 => embassy_stm32::spi::MODE_3,
        }
    }
}

#[derive(Copy, Clone)]
pub enum BitOrder {
    MsbFirst,
    LsbFirst,
}

impl From<BitOrder> for embassy_stm32::spi::BitOrder {
    fn from(bit_order: BitOrder) -> Self {
        match bit_order {
            BitOrder::MsbFirst => embassy_stm32::spi::BitOrder::MsbFirst,
            BitOrder::LsbFirst => embassy_stm32::spi::BitOrder::LsbFirst,
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
                    spim: InnerSpi<'static, Async>,
                }

                impl [<Spi $peripheral>] {
                    #[must_use]
                    pub fn new(
                        spim_peripheral:impl Peripheral<P = peripherals::$peripheral> + 'static,
                        sck_pin: impl Peripheral<P: SckPin<peripherals::$peripheral>> + 'static,
                        miso_pin: impl Peripheral<P: MisoPin<peripherals::$peripheral>> + 'static,
                        mosi_pin: impl Peripheral<P: MosiPin<peripherals::$peripheral>> + 'static,
                        tx_dma: impl Peripheral<P: TxDma<peripherals::$peripheral>> + 'static,
                        rx_dma: impl Peripheral<P: RxDma<peripherals::$peripheral>> + 'static,
                        config: Config,
                    ) -> Self {
                        let mut spi_config = embassy_stm32::spi::Config::default();
                        spi_config.frequency = config.frequency.into();
                        spi_config.mode = config.mode.into();
                        spi_config.bit_order = config.bit_order.into();
                        spi_config.miso_pull = gpio::Pull::None; // FIXME: ?

                        // The order of MOSI/MISO pins is inverted.
                        let spim = InnerSpi::new(
                            spim_peripheral,
                            sck_pin,
                            mosi_pin,
                            miso_pin,
                            tx_dma,
                            rx_dma,
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
                type Error = embassy_stm32::spi::Error;
            }

            impl_async_spibus_for_driver_enum!(Spi, $( $peripheral ),*);
        }
    };
}

// Define a driver per peripheral
riot_rs_macros::define_stm32_drivers!(Spi);
