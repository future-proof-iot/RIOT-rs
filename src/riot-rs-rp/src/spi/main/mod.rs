use embassy_embedded_hal::adapter::{BlockingAsync, YieldingAsync};
use embassy_rp::{
    peripherals,
    spi::{Blocking, ClkPin, MisoPin, MosiPin, Spi as InnerSpi},
    Peripheral,
};
use riot_rs_embassy_common::{
    impl_async_spibus_for_driver_enum,
    spi::{main::Kilohertz, Mode},
};

// TODO: we could consider making this `pub`
// NOTE(hal): values from the datasheets.
#[cfg(context = "rp2040")]
const MAX_FREQUENCY: Kilohertz = Kilohertz::kHz(62_500);

#[derive(Clone)]
#[non_exhaustive]
pub struct Config {
    pub frequency: Frequency,
    pub mode: Mode,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frequency: Frequency::F(Kilohertz::MHz(1)),
            mode: Mode::Mode0,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
pub enum Frequency {
    F(Kilohertz),
}

riot_rs_embassy_common::impl_spi_from_frequency!();
riot_rs_embassy_common::impl_spi_frequency_const_functions!(MAX_FREQUENCY);

impl Frequency {
    fn as_hz(&self) -> u32 {
        match self {
            Self::F(kilohertz) => kilohertz.to_Hz(),
        }
    }
}

macro_rules! define_spi_drivers {
    ($( $peripheral:ident ),* $(,)?) => {
        $(
            /// Peripheral-specific SPI driver.
            pub struct $peripheral {
                spim: YieldingAsync<BlockingAsync<InnerSpi<'static, peripherals::$peripheral, Blocking>>>,
            }

            impl $peripheral {
                #[expect(clippy::new_ret_no_self)]
                #[must_use]
                pub fn new(
                    sck_pin: impl Peripheral<P: ClkPin<peripherals::$peripheral>> + 'static,
                    miso_pin: impl Peripheral<P: MisoPin<peripherals::$peripheral>> + 'static,
                    mosi_pin: impl Peripheral<P: MosiPin<peripherals::$peripheral>> + 'static,
                    config: Config,
                ) -> Spi {
                    let (pol, phase) = crate::spi::from_mode(config.mode);

                    let mut spi_config = embassy_rp::spi::Config::default();
                    spi_config.frequency = config.frequency.as_hz();
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
                    let spi = InnerSpi::new_blocking(
                        spi_peripheral,
                        sck_pin,
                        mosi_pin,
                        miso_pin,
                        spi_config,
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
            type Error = embassy_rp::spi::Error;
        }

        impl_async_spibus_for_driver_enum!(Spi, $( $peripheral ),*);
    };
}

// Define a driver per peripheral
define_spi_drivers!(SPI0, SPI1);
