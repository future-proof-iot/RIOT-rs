use embassy_rp::{
    dma, peripherals,
    spi::{Async, ClkPin, MisoPin, MosiPin, Phase, Polarity, Spi as InnerSpi},
    Peripheral,
};

use crate::{
    arch,
    spi::{impl_async_spibus_for_driver_enum, Mode},
};

#[derive(Clone)]
#[non_exhaustive]
pub struct Config {
    pub frequency: Frequency,
    pub mode: Mode,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frequency: Frequency::_1M,
            mode: Mode::Mode0,
        }
    }
}

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum Frequency {
    _125k = 125_000,
    _250k = 250_000,
    _500k = 500_00,
    _1M = 1_000_000,
    _2M = 2_000_000,
    _4M = 4_000_000,
    _8M = 8_000_000,
    _16M = 16_000_000,
    _32M = 32_000_000,
}

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

pub(crate) fn init(peripherals: &mut arch::OptionalPeripherals) {
    // Take all SPI peripherals and do nothing with them.
    cfg_if::cfg_if! {
        if #[cfg(context = "rp2040")] {
            let _ = peripherals.SPI0.take().unwrap();
            let _ = peripherals.SPI1.take().unwrap();
        } else {
            compile_error!("this RP chip is not supported");
        }
    }
}

macro_rules! define_spi_drivers {
    ($( $peripheral:ident ),* $(,)?) => {
        $(
            /// Peripheral-specific SPI driver.
            pub struct $peripheral {
                spim: InnerSpi<'static, peripherals::$peripheral, Async>,
            }

            impl $peripheral {
                #[must_use]
                pub fn new(
                    sck_pin: impl Peripheral<P: ClkPin<peripherals::$peripheral>> + 'static,
                    miso_pin: impl Peripheral<P: MisoPin<peripherals::$peripheral>> + 'static,
                    mosi_pin: impl Peripheral<P: MosiPin<peripherals::$peripheral>> + 'static,
                    tx_dma: impl dma::Channel,
                    rx_dma: impl dma::Channel,
                    config: Config,
                ) -> Spi {
                    let (pol, phase) = config.mode.into();

                    let mut spi_config = embassy_rp::spi::Config::default();
                    spi_config.frequency = config.frequency as u32;
                    spi_config.polarity = pol;
                    spi_config.phase = phase;

                    // Make this struct a compile-time-enforced singleton: having multiple statics
                    // defined with the same name would result in a compile-time error.
                    paste::paste! {
                        #[allow(dead_code)]
                        static [<PREVENT_MULTIPLE_ $peripheral>]: () = ();
                    }

                    // FIXME(safety): enforce that the init code indeed has run
                    // SAFETY: this struct being a singleton prevents us from stealing the
                    // peripheral multiple times.
                    let spi_peripheral = unsafe { peripherals::$peripheral::steal() };

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

                    Spi::$peripheral(Self { spim: spi })
                }
            }
        )*

        /// Peripheral-agnostic driver.
        pub enum Spi {
            $( $peripheral($peripheral) ),*
        }

        impl embedded_hal_async::spi::ErrorType for Spi {
            type Error = embassy_rp::spi::Error;
        }

        impl_async_spibus_for_driver_enum!(Spi, $( $peripheral ),*);
    };
}

// Define a driver per peripheral
define_spi_drivers!(SPI0, SPI1);
