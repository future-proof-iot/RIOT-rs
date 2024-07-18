use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice as InnerSpiDevice;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

use crate::{arch, gpio};

// TODO: do we actually need a CriticalSectionRawMutex here?
pub type SpiDevice = InnerSpiDevice<'static, CriticalSectionRawMutex, arch::spi::Spi, gpio::Output>;

#[allow(unused_macros, reason = "used by arch modules")]
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
                    $( Self::$peripheral(spi) => SpiBus::flush(&mut spi.spim).await, )*
                }
            }
        }
    }
}
#[allow(unused_imports, reason = "used by arch modules")]
pub(crate) use impl_async_spibus_for_driver_enum;
