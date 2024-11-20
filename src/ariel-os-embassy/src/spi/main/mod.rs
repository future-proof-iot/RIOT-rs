//! Provides support for the SPI communication bus in main mode.

use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice as InnerSpiDevice;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

use crate::{gpio, hal};

pub use riot_rs_embassy_common::spi::main::*;

/// An SPI driver implementing [`embedded_hal_async::spi::SpiDevice`].
///
/// Needs to be provided with an MCU-specific SPI driver tied to a specific SPI peripheral,
/// obtainable from the [`hal::spi::main`] module.
/// It also requires a [`gpio::Output`] for the chip select (CS) signal.
///
/// See [`embedded_hal::spi`] to learn more about the distinction between an
/// [`SpiBus`](embedded_hal::spi::SpiBus) and an
/// [`SpiDevice`](embedded_hal::spi::SpiDevice).
///
/// # Note
///
/// Despite the driver interface being `async`, it may block during operations.
/// However, it cannot block indefinitely as a timeout is implemented, either by leveraging
/// SPI-specific hardware capabilities or through a generic software timeout.
// TODO: do we actually need a CriticalSectionRawMutex here?
pub type SpiDevice =
    InnerSpiDevice<'static, CriticalSectionRawMutex, hal::spi::main::Spi, gpio::Output>;

/// Returns the highest SPI frequency available on the MCU that fits into the requested
/// range.
///
/// # Examples
///
/// Assuming the MCU is only able to do up to 8 MHz:
///
/// ```
/// # use riot_rs_embassy::{hal, spi::main::{highest_freq_in, Kilohertz}};
/// let freq = const { highest_freq_in(Kilohertz::kHz(200)..=Kilohertz::MHz(16)) };
/// assert_eq!(freq, hal::spi::main::Frequency::F(Kilohertz::MHz(8)));
/// ```
///
/// # Panics
///
/// This function is only intended to be used in a `const` context.
/// It panics if no suitable frequency can be found.
pub const fn highest_freq_in(
    range: core::ops::RangeInclusive<riot_rs_embassy_common::spi::main::Kilohertz>,
) -> hal::spi::main::Frequency {
    let min = range.start().to_kHz();
    let max = range.end().to_kHz();

    assert!(max >= min);

    let mut freq = hal::spi::main::Frequency::first();

    loop {
        // If not yet in the requested range
        if freq.khz() < min {
            if let Some(next) = freq.next() {
                freq = next;
            } else {
                const_panic::concat_panic!(
                    "could not find a suitable SPI frequency: ",
                    min,
                    " kHz (minimum requested)",
                    " > ",
                    freq.khz(),
                    " kHz (highest available)"
                );
            }
        } else {
            break;
        }
    }

    loop {
        // If already outside of the requested range
        if freq.khz() > max {
            const_panic::concat_panic!(
                "could not find a suitable SPI frequency: ",
                max,
                " kHz (maximum requested) < ",
                freq.khz(),
                " kHz (lowest available)"
            );
        } else if let Some(next) = freq.next() {
            // The upper bound is inclusive.
            if next.khz() <= max {
                freq = next;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    freq
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_highest_freq_in() {
        use hal::spi::main::Frequency;
        use riot_rs_embassy_common::spi::main::Kilohertz;

        const FREQ_0: Frequency = highest_freq_in(Kilohertz::kHz(50)..=Kilohertz::kHz(150));
        const FREQ_1: Frequency = highest_freq_in(Kilohertz::kHz(100)..=Kilohertz::MHz(8));
        const FREQ_2: Frequency = highest_freq_in(Kilohertz::MHz(8)..=Kilohertz::MHz(10));

        assert_eq!(FREQ_0, Frequency::F(Kilohertz::kHz(150)));
        assert_eq!(FREQ_1, Frequency::F(Kilohertz::MHz(8)));
        assert_eq!(FREQ_2, Frequency::F(Kilohertz::MHz(8)));

        // FIXME: add another test to check when max < min
    }
}
