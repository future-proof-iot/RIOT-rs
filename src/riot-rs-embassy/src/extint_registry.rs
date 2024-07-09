pub use registry::{Error, ExtIntRegistry};

#[allow(dead_code, reason = "not used on all architectures")]
pub static EXTINT_REGISTRY: ExtIntRegistry = ExtIntRegistry::new();

#[cfg(context = "nrf")]
mod registry {
    use portable_atomic::{AtomicU8, Ordering};

    use crate::arch::{gpio::input::InputPin, peripheral::Peripheral};

    #[cfg(context = "nrf51")]
    const INT_CHANNEL_COUNT: u8 = 4;
    #[cfg(not(context = "nrf51"))]
    const INT_CHANNEL_COUNT: u8 = 8;

    pub struct ExtIntRegistry {
        used_interrupt_channel_count: AtomicU8,
    }

    impl ExtIntRegistry {
        #[must_use]
        pub const fn new() -> Self {
            Self {
                used_interrupt_channel_count: AtomicU8::new(0),
            }
        }

        pub fn use_interrupt_for_pin<PIN: Peripheral<P: InputPin>>(
            &self,
            _pin: &mut PIN, // Require the caller to have the peripheral
        ) -> Result<(), Error> {
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
                return Err(Error::NoIntChannelAvailable);
            }

            Ok(())
        }
    }

    // TODO: impl error-related traits?
    #[derive(Debug)]
    pub enum Error {
        NoIntChannelAvailable,
    }
}

#[cfg(not(context = "nrf"))]
mod registry {
    pub struct ExtIntRegistry {}

    impl ExtIntRegistry {
        #[must_use]
        pub const fn new() -> Self {
            Self {}
        }
    }

    #[derive(Debug)]
    pub enum Error {}
}
