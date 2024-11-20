use ariel_os_embassy_common::{
    impl_async_spibus_for_driver_enum,
    spi::{BitOrder, Mode},
};
use embassy_nrf::{
    bind_interrupts,
    gpio::Pin as GpioPin,
    peripherals,
    spim::{InterruptHandler, Spim},
    Peripheral,
};

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
            frequency: Frequency::_1M,
            mode: Mode::Mode0,
            bit_order: BitOrder::default(),
        }
    }
}

// NOTE(hal): limited set of frequencies available.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
pub enum Frequency {
    _125k,
    _250k,
    _500k,
    _1M,
    _2M,
    _4M,
    _8M,
    // FIXME(embassy): these frequencies are supported by hardware but do not seem supported by
    // Embassy.
    // #[cfg(any(context = "nrf52833", context = "nrf5340"))]
    // _16M,
    // #[cfg(any(context = "nrf52833", context = "nrf5340"))]
    // _32M,
}

impl Frequency {
    pub const fn first() -> Self {
        Self::_125k
    }

    pub const fn last() -> Self {
        Self::_8M
    }

    pub const fn next(self) -> Option<Self> {
        match self {
            Self::_125k => Some(Self::_250k),
            Self::_250k => Some(Self::_500k),
            Self::_500k => Some(Self::_1M),
            Self::_1M => Some(Self::_2M),
            Self::_2M => Some(Self::_4M),
            Self::_4M => Some(Self::_8M),
            Self::_8M => None,
        }
    }

    pub const fn prev(self) -> Option<Self> {
        match self {
            Self::_125k => None,
            Self::_250k => Some(Self::_125k),
            Self::_500k => Some(Self::_250k),
            Self::_1M => Some(Self::_500k),
            Self::_2M => Some(Self::_1M),
            Self::_4M => Some(Self::_2M),
            Self::_8M => Some(Self::_4M),
        }
    }

    pub const fn khz(self) -> u32 {
        match self {
            Self::_125k => 125,
            Self::_250k => 250,
            Self::_500k => 500,
            Self::_1M => 1000,
            Self::_2M => 2000,
            Self::_4M => 4000,
            Self::_8M => 8000,
        }
    }
}

impl From<ariel_os_embassy_common::spi::main::Frequency> for Frequency {
    fn from(freq: ariel_os_embassy_common::spi::main::Frequency) -> Self {
        match freq {
            ariel_os_embassy_common::spi::main::Frequency::_125k => Self::_125k,
            ariel_os_embassy_common::spi::main::Frequency::_250k => Self::_250k,
            ariel_os_embassy_common::spi::main::Frequency::_500k => Self::_500k,
            ariel_os_embassy_common::spi::main::Frequency::_1M => Self::_1M,
            ariel_os_embassy_common::spi::main::Frequency::_2M => Self::_2M,
            ariel_os_embassy_common::spi::main::Frequency::_4M => Self::_4M,
            ariel_os_embassy_common::spi::main::Frequency::_8M => Self::_8M,
        }
    }
}

impl From<Frequency> for embassy_nrf::spim::Frequency {
    fn from(freq: Frequency) -> Self {
        match freq {
            Frequency::_125k => embassy_nrf::spim::Frequency::K125,
            Frequency::_250k => embassy_nrf::spim::Frequency::K250,
            Frequency::_500k => embassy_nrf::spim::Frequency::K500,
            Frequency::_1M => embassy_nrf::spim::Frequency::M1,
            Frequency::_2M => embassy_nrf::spim::Frequency::M2,
            Frequency::_4M => embassy_nrf::spim::Frequency::M4,
            Frequency::_8M => embassy_nrf::spim::Frequency::M8,
        }
    }
}

macro_rules! define_spi_drivers {
    ($( $interrupt:ident => $peripheral:ident ),* $(,)?) => {
        $(
            /// Peripheral-specific SPI driver.
            pub struct $peripheral {
                spim: Spim<'static, peripherals::$peripheral>,
            }

            impl $peripheral {
                #[expect(clippy::new_ret_no_self)]
                #[must_use]
                pub fn new(
                    sck_pin: impl Peripheral<P: GpioPin> + 'static,
                    miso_pin: impl Peripheral<P: GpioPin> + 'static,
                    mosi_pin: impl Peripheral<P: GpioPin> + 'static,
                    config: Config,
                ) -> Spi {
                    let mut spi_config = embassy_nrf::spim::Config::default();
                    spi_config.frequency = config.frequency.into();
                    spi_config.mode = crate::spi::from_mode(config.mode);
                    spi_config.bit_order = crate::spi::from_bit_order(config.bit_order);

                    bind_interrupts!(
                        struct Irqs {
                            $interrupt => InterruptHandler<peripherals::$peripheral>;
                        }
                    );

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

                    let spim = Spim::new(
                        spim_peripheral,
                        Irqs,
                        sck_pin,
                        miso_pin,
                        mosi_pin,
                        spi_config,
                    );

                    Spi::$peripheral(Self { spim })
                }
            }
        )*

        /// Peripheral-agnostic driver.
        pub enum Spi {
            $( $peripheral($peripheral) ),*
        }

        impl embedded_hal_async::spi::ErrorType for Spi {
            type Error = embassy_nrf::spim::Error;
        }

        impl_async_spibus_for_driver_enum!(Spi, $( $peripheral ),*);
    };
}

// Define a driver per peripheral
#[cfg(context = "nrf52833")]
define_spi_drivers!(
    // FIXME: arbitrary selected peripherals
    // SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0 => TWISPI0,
    // SPIM1_SPIS1_TWIM1_TWIS1_SPI1_TWI1 => TWISPI1,
    // SPIM2_SPIS2_SPI2 => SPI2,
    SPIM3 => SPI3,
);
#[cfg(context = "nrf52840")]
define_spi_drivers!(
    // FIXME: arbitrary selected peripherals
    // SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0 => TWISPI0,
    // SPIM1_SPIS1_TWIM1_TWIS1_SPI1_TWI1 => TWISPI1,
    // SPIM2_SPIS2_SPI2 => SPI2,
    SPIM3 => SPI3,
);
// FIXME: arbitrary selected peripherals
#[cfg(context = "nrf5340")]
define_spi_drivers!(
    SERIAL2 => SERIAL2,
    SERIAL3 => SERIAL3,
);
