use embassy_embedded_hal::adapter::{BlockingAsync, YieldingAsync};
use esp_hal::{
    gpio::{self, InputPin, OutputPin},
    peripheral::Peripheral,
    peripherals,
    spi::{master::Spi as InnerSpi, FullDuplexMode},
};
use riot_rs_embassy_common::{
    impl_async_spibus_for_driver_enum,
    spi::{main::Kilohertz, BitOrder, Mode},
};

// TODO: we could consider making this `pub`
// NOTE(arch): values from the datasheets.
#[cfg(any(context = "esp32c3", context = "esp32c6"))]
const MAX_FREQUENCY: Kilohertz = Kilohertz::MHz(80);

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
            frequency: Frequency::F(Kilohertz::MHz(80)),
            mode: Mode::Mode0,
            bit_order: BitOrder::default(),
        }
    }
}

// Possible values are copied from embassy-nrf
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
pub enum Frequency {
    F(Kilohertz),
}

riot_rs_embassy_common::impl_spi_from_frequency!();
riot_rs_embassy_common::impl_spi_frequency_const_functions!(MAX_FREQUENCY);

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
                spim: YieldingAsync<BlockingAsync<InnerSpi<'static, peripherals::$peripheral, FullDuplexMode>>>,
            }

            impl $peripheral {
                #[expect(clippy::new_ret_no_self)]
                #[must_use]
                pub fn new(
                    sck_pin: impl Peripheral<P: OutputPin> + 'static,
                    miso_pin: impl Peripheral<P: InputPin> + 'static,
                    mosi_pin: impl Peripheral<P: OutputPin> + 'static,
                    config: Config,
                ) -> Spi {
                    let frequency = config.frequency.into();

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

                    let spi = esp_hal::spi::master::Spi::new(
                        spi_peripheral,
                        frequency,
                        crate::spi::from_mode(config.mode),
                    );
                    let spi = spi.with_bit_order(
                        crate::spi::from_bit_order(config.bit_order), // Read order
                        crate::spi::from_bit_order(config.bit_order), // Write order
                    );
                    // The order of MOSI/MISO pins is inverted.
                    let spi = spi.with_pins(
                        sck_pin,
                        mosi_pin,
                        miso_pin,
                        gpio::NoPin, // The CS pin is managed separately
                    );

                    Spi::$peripheral(Self { spim: YieldingAsync::new(BlockingAsync::new(spi)) })
                }
            }
        )*

        /// Peripheral-agnostic driver.
        pub enum Spi {
            $( $peripheral($peripheral) ),*
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
