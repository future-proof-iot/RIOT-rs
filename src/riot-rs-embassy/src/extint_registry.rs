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

#[cfg(context = "stm32")]
mod registry {
    use embassy_stm32::exti::{AnyChannel, Channel};
    use portable_atomic::{AtomicBool, AtomicU16, Ordering};

    use crate::arch::{self, gpio::input::InputPin, peripheral::Peripheral, peripherals};

    pub struct ExtIntRegistry {
        initialized: AtomicBool,
        used_interrupt_channels: AtomicU16, // 16 channels
    }

    impl ExtIntRegistry {
        // Collect all channel peripherals so that the registry is the only one managing them.
        pub const fn new() -> Self {
            Self {
                initialized: AtomicBool::new(false),
                used_interrupt_channels: AtomicU16::new(0),
            }
        }

        pub fn init(&self, peripherals: &mut arch::OptionalPeripherals) {
            peripherals.EXTI0.take().unwrap();
            peripherals.EXTI1.take().unwrap();
            peripherals.EXTI2.take().unwrap();
            peripherals.EXTI3.take().unwrap();
            peripherals.EXTI4.take().unwrap();
            peripherals.EXTI5.take().unwrap();
            peripherals.EXTI6.take().unwrap();
            peripherals.EXTI7.take().unwrap();
            peripherals.EXTI8.take().unwrap();
            peripherals.EXTI9.take().unwrap();
            peripherals.EXTI10.take().unwrap();
            peripherals.EXTI11.take().unwrap();
            peripherals.EXTI12.take().unwrap();
            peripherals.EXTI13.take().unwrap();
            peripherals.EXTI14.take().unwrap();
            peripherals.EXTI15.take().unwrap();

            self.initialized.store(true, Ordering::Release);

            // Do nothing else, just consume the peripherals: they are ours now!
        }

        pub fn get_interrupt_channel_for_pin<P: Peripheral<P = T>, T: InputPin>(
            &self,
            pin: P,
        ) -> Result<AnyChannel, Error> {
            // Make sure that the interrupt channels have been captured during initialization.
            assert!(self.initialized.load(Ordering::Acquire));

            let pin = pin.into_ref().map_into();
            let pin_number = pin.pin();

            // As interrupt channels are mutually exclusive between ports (ie., if channel i has
            // been bound for pin i of a port, it cannot be used for pin i of another port), we
            // only check the pin number.
            // NOTE(ordering): since setting a bit is an idempotent operation, and since we do not
            // allow clearing them, the ordering does not matter.
            let was_used = self
                .used_interrupt_channels
                .bit_set(pin_number.into(), Ordering::Relaxed);

            if was_used {
                return Err(Error::IntChannelAlreadyUsed);
            }

            // They are the same
            let ch_number = pin_number;

            // NOTE(embassy): ideally we would be using `T::ExtiChannel::steal()` instead of this
            // match, but Embassy does not provide this.
            // SAFETY: this function enforces that the same channel cannot be obtained twice,
            // making sure multiple instances are not used at the same time as the mandatory
            // `init()` method has collected all channel peripherals beforehand.
            let ch = match ch_number {
                0 => unsafe { peripherals::EXTI0::steal() }.degrade(),
                1 => unsafe { peripherals::EXTI1::steal() }.degrade(),
                2 => unsafe { peripherals::EXTI2::steal() }.degrade(),
                3 => unsafe { peripherals::EXTI3::steal() }.degrade(),
                4 => unsafe { peripherals::EXTI4::steal() }.degrade(),
                5 => unsafe { peripherals::EXTI5::steal() }.degrade(),
                6 => unsafe { peripherals::EXTI6::steal() }.degrade(),
                7 => unsafe { peripherals::EXTI7::steal() }.degrade(),
                8 => unsafe { peripherals::EXTI8::steal() }.degrade(),
                9 => unsafe { peripherals::EXTI9::steal() }.degrade(),
                10 => unsafe { peripherals::EXTI10::steal() }.degrade(),
                11 => unsafe { peripherals::EXTI11::steal() }.degrade(),
                12 => unsafe { peripherals::EXTI12::steal() }.degrade(),
                13 => unsafe { peripherals::EXTI13::steal() }.degrade(),
                14 => unsafe { peripherals::EXTI14::steal() }.degrade(),
                15 => unsafe { peripherals::EXTI15::steal() }.degrade(),
                _ => unreachable!(),
            };

            Ok(ch)
        }
    }

    // TODO: impl error-related traits?
    #[derive(Debug)]
    pub enum Error {
        IntChannelAlreadyUsed,
    }
}

#[cfg(not(any(context = "nrf", context = "stm32")))]
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
