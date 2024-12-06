//! Provides support for the SPI communication bus in main mode.

use ariel_os_embassy_common::{
    impl_async_spibus_for_driver_enum,
    spi::{main::Kilohertz, BitOrder, Mode},
};
use embassy_embedded_hal::adapter::{BlockingAsync, YieldingAsync};
use esp_hal::{
    gpio::{self, interconnect::PeripheralOutput},
    peripheral::Peripheral,
    peripherals,
    spi::master::Spi as InnerSpi,
};

// TODO: we could consider making this `pub`
// NOTE(hal): values from the datasheets.
#[cfg(any(context = "esp32c3", context = "esp32c6"))]
const MAX_FREQUENCY: Kilohertz = Kilohertz::MHz(80);

/// SPI bus configuration.
#[derive(Clone)]
#[non_exhaustive]
pub struct Config {
    /// The frequency at which the bus should operate.
    pub frequency: Frequency,
    /// The SPI mode to use.
    pub mode: Mode,
    #[doc(hidden)]
    pub bit_order: BitOrder,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frequency: Frequency::F(Kilohertz::MHz(80)),
            mode: Mode::Mode0,
            bit_order: BitOrder::default(),
        }
    }
}

/// SPI bus frequency.
// Possible values are copied from embassy-nrf
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
pub enum Frequency {
    /// Arbitrary frequency.
    F(Kilohertz),
}

ariel_os_embassy_common::impl_spi_from_frequency!();
ariel_os_embassy_common::impl_spi_frequency_const_functions!(MAX_FREQUENCY);

impl From<Frequency> for fugit::HertzU32 {
    fn from(freq: Frequency) -> Self {
        match freq {
            Frequency::F(kilohertz) => fugit::HertzU32::kHz(kilohertz.to_kHz()),
        }
    }
}

macro_rules! define_spi_drivers {
    ($( $peripheral:ident ),* $(,)?) => {
        $(
            /// Peripheral-specific SPI driver.
            pub struct $peripheral {
                spim: YieldingAsync<BlockingAsync<InnerSpi<'static, esp_hal::Blocking>>>,
            }

            impl $peripheral {
                /// Returns a driver implementing [`embedded_hal_async::spi::SpiBus`] for this SPI
                /// peripheral.
                #[expect(clippy::new_ret_no_self)]
                #[must_use]
                pub fn new(
                    sck_pin: impl Peripheral<P: PeripheralOutput> + 'static,
                    miso_pin: impl Peripheral<P: PeripheralOutput> + 'static,
                    mosi_pin: impl Peripheral<P: PeripheralOutput> + 'static,
                    config: Config,
                ) -> Spi {
                    let mut spi_config = esp_hal::spi::master::Config::default();
                    spi_config.frequency = config.frequency.into();
                    spi_config.mode = crate::spi::from_mode(config.mode);
                    spi_config.read_bit_order = crate::spi::from_bit_order(config.bit_order);
                    spi_config.write_bit_order = crate::spi::from_bit_order(config.bit_order);

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

                    let spi = esp_hal::spi::master::Spi::new_with_config(
                        spi_peripheral,
                        spi_config,
                    )
                        .with_sck(sck_pin)
                        .with_mosi(mosi_pin)
                        .with_miso(miso_pin)
                        .with_cs(gpio::NoPin); // The CS pin is managed separately

                    Spi::$peripheral(Self { spim: YieldingAsync::new(BlockingAsync::new(spi)) })
                }
            }
        )*

        /// Peripheral-agnostic driver.
        pub enum Spi {
            $(
                #[doc = concat!(stringify!($peripheral), " peripheral.")]
                $peripheral($peripheral)
            ),*
        }

        impl embedded_hal_async::spi::ErrorType for Spi {
            type Error = esp_hal::spi::Error;
        }

        impl_async_spibus_for_driver_enum!(Spi, $( $peripheral ),*);
    };
}

// FIXME: there seems to be an SPI3 on ESP32-S2 and ESP32-S3
// Define a driver per peripheral
#[cfg(context = "esp32c6")]
define_spi_drivers!(SPI2);
