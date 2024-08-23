use embassy_stm32::{
    gpio,
    mode::Async,
    peripherals,
    spi::{MisoPin, MosiPin, RxDma, SckPin, Spi as InnerSpi, TxDma},
    time::Hertz,
    Peripheral,
};
use riot_rs_macros::call_with_stm32_peripheral_list;

use crate::{
    arch,
    spi::{impl_async_spibus_for_driver_enum, BitOrder, Mode},
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

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum Frequency {
    _125k,
    _250k,
    _500k,
    _1M,
    _2M,
    _4M,
    _8M,
    _16M,
    _32M,
}

impl From<Frequency> for Hertz {
    fn from(freq: Frequency) -> Self {
        match freq {
            Frequency::_125k => Hertz::khz(125),
            Frequency::_250k => Hertz::khz(250),
            Frequency::_500k => Hertz::khz(500),
            Frequency::_1M => Hertz::mhz(1),
            Frequency::_2M => Hertz::mhz(2),
            Frequency::_4M => Hertz::mhz(4),
            Frequency::_8M => Hertz::mhz(8),
            Frequency::_16M => Hertz::mhz(16),
            Frequency::_32M => Hertz::mhz(32),
        }
    }
}

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

impl From<BitOrder> for embassy_stm32::spi::BitOrder {
    fn from(bit_order: BitOrder) -> Self {
        match bit_order {
            BitOrder::MsbFirst => embassy_stm32::spi::BitOrder::MsbFirst,
            BitOrder::LsbFirst => embassy_stm32::spi::BitOrder::LsbFirst,
        }
    }
}

pub(crate) fn init(peripherals: &mut arch::OptionalPeripherals) {
    // This macro has to be defined in this function so that the `peripherals` variables exists.
    macro_rules! take_all_spi_peripherals {
        ($peripherals:ident, $( $peripheral:ident ),*) => {
            $(
                let _ = peripherals.$peripheral.take().unwrap();
            )*
        }
    }

    // Take all SPI peripherals and do nothing with them.
    call_with_stm32_peripheral_list!(take_all_spi_peripherals!, Spi, Peripherals);
}

macro_rules! define_spi_drivers {
    ($( $interrupt:ident => $peripheral:ident ),* $(,)?) => {
        $(
            /// Peripheral-specific SPI driver.
            pub struct $peripheral {
                spim: InnerSpi<'static, Async>,
            }

            impl $peripheral {
                #[must_use]
                pub fn new(
                    sck_pin: impl Peripheral<P: SckPin<peripherals::$peripheral>> + 'static,
                    miso_pin: impl Peripheral<P: MisoPin<peripherals::$peripheral>> + 'static,
                    mosi_pin: impl Peripheral<P: MosiPin<peripherals::$peripheral>> + 'static,
                    tx_dma: impl Peripheral<P: TxDma<peripherals::$peripheral>> + 'static,
                    rx_dma: impl Peripheral<P: RxDma<peripherals::$peripheral>> + 'static,
                    config: Config,
                ) -> Spi {
                    let mut spi_config = embassy_stm32::spi::Config::default();
                    spi_config.frequency = config.frequency.into();
                    spi_config.mode = config.mode.into();
                    spi_config.bit_order = config.bit_order.into();
                    spi_config.miso_pull = gpio::Pull::None; // FIXME: ?

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
                    let spim = InnerSpi::new(
                        spim_peripheral,
                        sck_pin,
                        mosi_pin,
                        miso_pin,
                        tx_dma,
                        rx_dma,
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
            type Error = embassy_stm32::spi::Error;
        }

        impl_async_spibus_for_driver_enum!(Spi, $( $peripheral ),*);
    };
}

// Define a driver per peripheral
call_with_stm32_peripheral_list!(define_spi_drivers!, Spi, PeripheralsAndInterrupts);
