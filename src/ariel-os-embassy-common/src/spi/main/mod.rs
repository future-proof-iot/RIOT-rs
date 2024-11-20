//! Provides HAL-agnostic SPI-related types, for main mode.

pub use fugit::KilohertzU32 as Kilohertz;

// FIXME: rename this to Bitrate and use bps instead?
/// SPI bus frequencies supported on all MCUs.
#[derive(Copy, Clone)]
pub enum Frequency {
    /// 125 kHz.
    _125k,
    /// 250 kHz.
    _250k,
    /// 500 kHz.
    _500k,
    /// 1 MHz.
    _1M,
    /// 2 MHz.
    _2M,
    /// 4 MHz.
    _4M,
    /// 8 MHz.
    _8M,
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_spi_from_frequency {
    () => {
        impl From<ariel_os_embassy_common::spi::main::Frequency> for Frequency {
            fn from(freq: ariel_os_embassy_common::spi::main::Frequency) -> Self {
                match freq {
                    ariel_os_embassy_common::spi::main::Frequency::_125k => {
                        Self::F($crate::spi::main::Kilohertz::kHz(125))
                    }
                    ariel_os_embassy_common::spi::main::Frequency::_250k => {
                        Self::F($crate::spi::main::Kilohertz::kHz(250))
                    }
                    ariel_os_embassy_common::spi::main::Frequency::_500k => {
                        Self::F($crate::spi::main::Kilohertz::kHz(500))
                    }
                    ariel_os_embassy_common::spi::main::Frequency::_1M => {
                        Self::F($crate::spi::main::Kilohertz::MHz(1))
                    }
                    ariel_os_embassy_common::spi::main::Frequency::_2M => {
                        Self::F($crate::spi::main::Kilohertz::MHz(2))
                    }
                    ariel_os_embassy_common::spi::main::Frequency::_4M => {
                        Self::F($crate::spi::main::Kilohertz::MHz(4))
                    }
                    ariel_os_embassy_common::spi::main::Frequency::_8M => {
                        Self::F($crate::spi::main::Kilohertz::MHz(8))
                    }
                }
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_spi_frequency_const_functions {
    ($MAX_FREQUENCY:ident) => {
        #[doc(hidden)]
        impl Frequency {
            pub const fn first() -> Self {
                Self::F(Kilohertz::kHz(1))
            }

            pub const fn last() -> Self {
                Self::F(MAX_FREQUENCY)
            }

            pub const fn next(self) -> Option<Self> {
                match self {
                    Self::F(kilohertz) => {
                        let khz = kilohertz.to_kHz();
                        if khz < MAX_FREQUENCY.to_kHz() {
                            Some(Self::F(Kilohertz::kHz(khz + 1)))
                        } else {
                            None
                        }
                    }
                }
            }

            pub const fn prev(self) -> Option<Self> {
                const MIN_FREQUENCY: Kilohertz = Kilohertz::kHz(1);

                match self {
                    Self::F(kilohertz) => {
                        let khz = kilohertz.to_kHz();
                        if khz > MIN_FREQUENCY.to_kHz() {
                            Some(Self::F(Kilohertz::kHz(khz - 1)))
                        } else {
                            None
                        }
                    }
                }
            }

            pub const fn khz(self) -> u32 {
                match self {
                    Self::F(kilohertz) => kilohertz.to_kHz(),
                }
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_async_spibus_for_driver_enum {
    ($driver_enum:ident, $( $peripheral:ident ),*) => {
        // The `SpiBus` trait represents exclusive ownership over the whole bus.
        impl embedded_hal_async::spi::SpiBus for $driver_enum {
            async fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
                match self {
                    $( Self::$peripheral(spi) => spi.spim.read(words).await, )*
                }
            }

            async fn write(&mut self, data: &[u8]) -> Result<(), Self::Error> {
                match self {
                    $( Self::$peripheral(spi) => spi.spim.write(data).await, )*
                }
            }

            async fn transfer(&mut self, rx: &mut [u8], tx: &[u8]) -> Result<(), Self::Error> {
                match self {
                    $( Self::$peripheral(spi) => spi.spim.transfer(rx, tx).await, )*
                }
            }

            async fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
                match self {
                    $( Self::$peripheral(spi) => spi.spim.transfer_in_place(words).await, )*
                }
            }

            async fn flush(&mut self) -> Result<(), Self::Error> {
                use embedded_hal_async::spi::SpiBus;
                match self {
                    $( Self::$peripheral(spi) => SpiBus::<u8>::flush(&mut spi.spim).await, )*
                }
            }
        }
    }
}
