//! Provides support for the I2C communication bus in controller mode.

use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice as InnerI2cDevice;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

use crate::arch;

pub use riot_rs_embassy_common::{i2c::controller::*, kHz};

/// An I2C driver implementing [`embedded_hal_async::i2c::I2c`].
///
/// It needs to be provided with an MCU-specific I2C driver tied to a specific I2C peripheral,
/// obtained as [`arch::i2c::controller::I2c`].
///
/// See [`embedded_hal::i2c`] to learn more about how to share the bus.
///
/// # Note
///
/// Despite the driver interface being `async`, it may block during operations.
/// However, it cannot block indefinitely as a timeout is implemented, either by leveraging
/// I2C-specific hardware capabilities or through a generic software timeout.
// TODO: do we actually need a CriticalSectionRawMutex here?
pub type I2cDevice = InnerI2cDevice<'static, CriticalSectionRawMutex, arch::i2c::controller::I2c>;

/// Returns the highest I2C frequency available on the architecture that fits into the requested
/// range.
///
/// # Examples
///
/// Assuming the architecture is only able to do 100 kHz and 400 kHz (not 250 kHz):
///
/// ```
/// # use riot_rs_embassy::{arch, i2c::controller::{highest_freq_in, kHz}};
/// let freq = const { highest_freq_in(kHz(100)..=kHz(250)) };
/// assert_eq!(freq, arch::i2c::controller::Frequency::_100k);
/// ```
///
/// # Panics
///
/// This function is only intended to be used in a `const` context.
/// It panics if no suitable frequency can be found.
pub const fn highest_freq_in(
    range: core::ops::RangeInclusive<riot_rs_embassy_common::kHz>,
) -> arch::i2c::controller::Frequency {
    let min = range.start().0;
    let max = range.end().0;

    assert!(max >= min);

    let mut freq = arch::i2c::controller::Frequency::first();

    loop {
        // If not yet in the requested range
        if freq.khz() < min {
            if let Some(next) = freq.next() {
                freq = next;
            } else {
                const_panic::concat_panic!(
                    "could not find a suitable I2C frequency: ",
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
                "could not find a suitable I2C frequency: ",
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

    return freq;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_highest_freq_in() {
        const FREQ_0: arch::i2c::controller::Frequency = highest_freq_in(kHz(50)..=kHz(150));
        const FREQ_1: arch::i2c::controller::Frequency = highest_freq_in(kHz(100)..=kHz(100));
        const FREQ_2: arch::i2c::controller::Frequency = highest_freq_in(kHz(50)..=kHz(100));
        const FREQ_3: arch::i2c::controller::Frequency = highest_freq_in(kHz(50)..=kHz(400));
        const FREQ_4: arch::i2c::controller::Frequency = highest_freq_in(kHz(100)..=kHz(400));
        const FREQ_5: arch::i2c::controller::Frequency = highest_freq_in(kHz(300)..=kHz(400));
        const FREQ_6: arch::i2c::controller::Frequency = highest_freq_in(kHz(100)..=kHz(450));
        const FREQ_7: arch::i2c::controller::Frequency = highest_freq_in(kHz(300)..=kHz(450));

        // The only available values in the dummy arch are 100k and 400k.
        assert_eq!(FREQ_0, arch::i2c::controller::Frequency::_100k);
        assert_eq!(FREQ_1, arch::i2c::controller::Frequency::_100k);
        assert_eq!(FREQ_2, arch::i2c::controller::Frequency::_100k);
        assert_eq!(FREQ_3, arch::i2c::controller::Frequency::_400k);
        assert_eq!(FREQ_4, arch::i2c::controller::Frequency::_400k);
        assert_eq!(FREQ_5, arch::i2c::controller::Frequency::_400k);
        assert_eq!(FREQ_6, arch::i2c::controller::Frequency::_400k);
        assert_eq!(FREQ_7, arch::i2c::controller::Frequency::_400k);

        // FIXME: add another test to check when max < min
        // and with
        // const FREQ_0: arch::i2c::controller::Frequency = highest_freq_in(kHz(50)..=kHz(80));
    }
}
