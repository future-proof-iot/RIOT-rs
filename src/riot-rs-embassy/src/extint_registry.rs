pub use registry::{Error, ExtIntRegistry};

#[allow(dead_code, reason = "not used on all architectures")]
pub static EXTINT_REGISTRY: ExtIntRegistry = ExtIntRegistry::new();

#[cfg(context = "nrf")]
mod registry {
    use portable_atomic::{AtomicU8, Ordering};

    use crate::arch::{gpio::input::Pin, peripheral::Peripheral};

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

        pub fn use_interrupt_for_pin<PIN: Peripheral<P: Pin>>(
            &self,
            pin: PIN,
        ) -> Result<PIN, Error> {
            // TODO: check and justify the ordering
            // There is less than `u8::MAX` interrupt channels, so wrapping around on overflow is
            // not a concern.
            let prev_used_interrupt_channel_count = self
                .used_interrupt_channel_count
                .fetch_add(1, Ordering::AcqRel);

            // Testing for equality because this is the value *before* being incremented.
            if prev_used_interrupt_channel_count >= INT_CHANNEL_COUNT {
                self.used_interrupt_channel_count.sub(1, Ordering::AcqRel);
                return Err(Error::NoIntChannelAvailable);
            }

            Ok(pin)
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
