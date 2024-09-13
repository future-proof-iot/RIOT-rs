use embassy_nrf::{gpio::Pin, Peripheral};
use portable_atomic::{AtomicU8, Ordering};
use riot_rs_embassy_common::gpio::input::InterruptError;

#[cfg(context = "nrf51")]
const INT_CHANNEL_COUNT: u8 = 4;
#[cfg(not(context = "nrf51"))]
const INT_CHANNEL_COUNT: u8 = 8;

pub static EXTINT_REGISTRY: ExtIntRegistry = ExtIntRegistry::new();

pub struct ExtIntRegistry {
    used_interrupt_channel_count: AtomicU8,
}

impl ExtIntRegistry {
    #[expect(clippy::new_without_default)]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            used_interrupt_channel_count: AtomicU8::new(0),
        }
    }

    pub fn use_interrupt_for_pin<PIN: Peripheral<P: Pin>>(
        &self,
        _pin: &mut PIN, // Require the caller to have the peripheral
    ) -> Result<(), InterruptError> {
        // NOTE(ordering): this acts as a lock, so we use Acquire/Release ordering.
        let update_res = self.used_interrupt_channel_count.fetch_update(
            Ordering::AcqRel,
            Ordering::Acquire,
            |c| {
                if c == INT_CHANNEL_COUNT {
                    None
                } else {
                    // This cannot overflow because `INT_CHANNEL_COUNT` is lower than u8::MAX.
                    Some(c + 1)
                }
            },
        );

        if update_res.is_err() {
            return Err(InterruptError::NoIntChannelAvailable);
        }

        Ok(())
    }
}
