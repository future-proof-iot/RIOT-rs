use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice as InnerSpiDevice;
use embassy_rp::{
    dma, gpio, peripherals,
    spi::{Async, ClkPin, MisoPin, MosiPin, Phase, Polarity, Spi as InnerSpi},
};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

use crate::spi::impl_async_spibus_for_driver_enum;

// TODO: factor this out across archs?
pub type SpiDevice = InnerSpiDevice<'static, CriticalSectionRawMutex, Spi, gpio::Output<'static>>;

#[derive(Clone)]
#[non_exhaustive]
pub struct Config {
    pub frequency: Frequency,
    pub mode: Mode,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frequency: Frequency::M1,
            mode: Mode::Mode0,
        }
    }
}

// Possible values are copied from embassy-nrf
// TODO: check how well this matches the RP2040 capabilities
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

#[derive(Copy, Clone)]
pub enum Mode {
    Mode0,
    Mode1,
    Mode2,
    Mode3,
}

// https://en.wikipedia.org/wiki/Serial_Peripheral_Interface#Mode_numbers
impl From<Mode> for (Polarity, Phase) {
    fn from(mode: Mode) -> Self {
        match mode {
            Mode::Mode0 => (Polarity::IdleLow, Phase::CaptureOnFirstTransition),
            Mode::Mode1 => (Polarity::IdleLow, Phase::CaptureOnSecondTransition),
            Mode::Mode2 => (Polarity::IdleHigh, Phase::CaptureOnFirstTransition),
            Mode::Mode3 => (Polarity::IdleHigh, Phase::CaptureOnSecondTransition),
        }
    }
}

macro_rules! define_spi_drivers {
    ($( $peripheral:ident ),* $(,)?) => {
        // paste allows to create new identifiers by concatenation using `[<foo bar>]`.
        paste::paste! {
            $(
                pub struct [<Spi $peripheral>] {
                    spim: InnerSpi<'static, peripherals::$peripheral, Async>,
                }

                impl [<Spi $peripheral>] {
                    #[must_use]
                    pub fn new(
                        spi_peripheral: peripherals::$peripheral,
                        sck_pin: impl ClkPin<peripherals::$peripheral>,
                        miso_pin: impl MisoPin<peripherals::$peripheral>,
                        mosi_pin: impl MosiPin<peripherals::$peripheral>,
                        tx_dma: impl dma::Channel,
                        rx_dma: impl dma::Channel,
                        config: Config,
                    ) -> Self {
                        let (pol, phase) = config.mode.into();

                        let mut spi_config = embassy_rp::spi::Config::default();
                        spi_config.frequency = config.frequency as u32;
                        spi_config.polarity = pol;
                        spi_config.phase = phase;

                        // The order of MOSI/MISO pins is inverted.
                        let spi = InnerSpi::new(
                            spi_peripheral,
                            sck_pin,
                            mosi_pin,
                            miso_pin,
                            tx_dma,
                            rx_dma,
                            spi_config,
                        );

                        Self { spim: spi }
                    }
                }
            )*

            // Each enum variant is for a specific peripheral.
            pub enum Spi {
                $( $peripheral([<Spi $peripheral>]), )*
            }

            impl embedded_hal_async::spi::ErrorType for Spi {
                type Error = embassy_rp::spi::Error;
            }

            impl_async_spibus_for_driver_enum!(Spi, $( $peripheral ),*);
        }
    };
}

// Define a driver per peripheral
define_spi_drivers!(SPI0, SPI1);
