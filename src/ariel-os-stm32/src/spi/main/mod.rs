use embassy_embedded_hal::adapter::{BlockingAsync, YieldingAsync};
use embassy_stm32::{
    gpio,
    mode::Blocking,
    peripherals,
    spi::{MisoPin, MosiPin, SckPin, Spi as InnerSpi},
    time::Hertz,
    Peripheral,
};
use ariel_os_embassy_common::{
    impl_async_spibus_for_driver_enum,
    spi::{main::Kilohertz, BitOrder, Mode},
};

// TODO: we could consider making this `pub`
// NOTE(hal): values from the datasheets.
// When peripherals support different frequencies, the smallest one is used.
#[cfg(context = "stm32f401retx")]
const MAX_FREQUENCY: Kilohertz = Kilohertz::MHz(21);
#[cfg(context = "stm32h755zitx")]
const MAX_FREQUENCY: Kilohertz = Kilohertz::MHz(150);
#[cfg(context = "stm32wb55rgvx")]
const MAX_FREQUENCY: Kilohertz = Kilohertz::MHz(32);

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
            frequency: Frequency::F(Kilohertz::MHz(1)),
            mode: Mode::Mode0,
            bit_order: BitOrder::default(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
pub enum Frequency {
    F(Kilohertz),
}

impl From<Frequency> for Hertz {
    fn from(freq: Frequency) -> Self {
        match freq {
            Frequency::F(kilohertz) => Hertz::khz(kilohertz.to_kHz()),
        }
    }
}

ariel_os_embassy_common::impl_spi_from_frequency!();
ariel_os_embassy_common::impl_spi_frequency_const_functions!(MAX_FREQUENCY);

macro_rules! define_spi_drivers {
    ($( $interrupt:ident => $peripheral:ident ),* $(,)?) => {
        $(
            /// Peripheral-specific SPI driver.
            pub struct $peripheral {
                spim: YieldingAsync<BlockingAsync<InnerSpi<'static, Blocking>>>,
            }

            impl $peripheral {
                #[expect(clippy::new_ret_no_self)]
                #[must_use]
                pub fn new(
                    sck_pin: impl Peripheral<P: SckPin<peripherals::$peripheral>> + 'static,
                    miso_pin: impl Peripheral<P: MisoPin<peripherals::$peripheral>> + 'static,
                    mosi_pin: impl Peripheral<P: MosiPin<peripherals::$peripheral>> + 'static,
                    config: Config,
                ) -> Spi {
                    let mut spi_config = embassy_stm32::spi::Config::default();
                    spi_config.frequency = config.frequency.into();
                    spi_config.mode = crate::spi::from_mode(config.mode);
                    spi_config.bit_order = crate::spi::from_bit_order(config.bit_order);
                    spi_config.miso_pull = gpio::Pull::None;

                    // Make this struct a compile-time-enforced singleton: having multiple statics
                    // defined with the same name would result in a compile-time error.
                    paste::paste! {
                        #[allow(dead_code)]
                        static [<PREVENT_MULTIPLE_ $peripheral>]: () = ();
                    }

                    // FIXME(safety): enforce that the init code indeed has run
                    // SAFETY: this struct being a singleton prevents us from stealing the
                    // peripheral multiple times.
                    let spim_peripheral = unsafe { peripherals::$peripheral::steal() };

                    // The order of MOSI/MISO pins is inverted.
                    let spim = InnerSpi::new_blocking(
                        spim_peripheral,
                        sck_pin,
                        mosi_pin,
                        miso_pin,
                        spi_config,
                    );

                    Spi::$peripheral(Self { spim: YieldingAsync::new(BlockingAsync::new(spim)) })
                }
            }
        )*

        /// Peripheral-agnostic driver.
        pub enum Spi {
            $( $peripheral($peripheral) ),*
        }

        impl embedded_hal_async::spi::ErrorType for Spi {
            type Error = embassy_stm32::spi::Error;
        }

        impl_async_spibus_for_driver_enum!(Spi, $( $peripheral ),*);
    };
}

// Define a driver per peripheral
#[cfg(context = "stm32f401retx")]
define_spi_drivers!(
   SPI1 => SPI1,
   SPI2 => SPI2,
   SPI3 => SPI3,
);
#[cfg(context = "stm32h755zitx")]
define_spi_drivers!(
   SPI1 => SPI1,
   SPI2 => SPI2,
   SPI3 => SPI3,
   SPI4 => SPI4,
   SPI5 => SPI5,
   SPI6 => SPI6,
);
#[cfg(context = "stm32wb55rgvx")]
define_spi_drivers!(
   SPI1 => SPI1,
   SPI2 => SPI2,
);
