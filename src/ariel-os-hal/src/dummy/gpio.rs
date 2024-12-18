macro_rules! define_input_like {
    ($type:ident) => {
        pub struct $type<'d> {
            _marker: core::marker::PhantomData<&'d ()>,
        }

        impl $type<'_> {
            #[must_use]
            pub fn is_high(&self) -> bool {
                unimplemented!();
            }

            #[must_use]
            pub fn is_low(&self) -> bool {
                unimplemented!();
            }

            #[must_use]
            pub fn get_level(&self) -> crate::gpio::input::Level {
                unimplemented!();
            }

            pub async fn wait_for_high(&mut self) {
                unimplemented!();
            }

            pub async fn wait_for_low(&mut self) {
                unimplemented!();
            }

            pub async fn wait_for_rising_edge(&mut self) {
                unimplemented!();
            }

            pub async fn wait_for_falling_edge(&mut self) {
                unimplemented!();
            }

            pub async fn wait_for_any_edge(&mut self) {
                unimplemented!();
            }
        }

        impl embedded_hal::digital::ErrorType for $type<'_> {
            type Error = core::convert::Infallible;
        }

        impl embedded_hal::digital::InputPin for $type<'_> {
            fn is_low(&mut self) -> Result<bool, Self::Error> {
                unimplemented!();
            }

            fn is_high(&mut self) -> Result<bool, Self::Error> {
                unimplemented!();
            }
        }

        impl embedded_hal_async::digital::Wait for $type<'_> {
            async fn wait_for_high(&mut self) -> Result<(), Self::Error> {
                unimplemented!();
            }

            async fn wait_for_low(&mut self) -> Result<(), Self::Error> {
                unimplemented!();
            }

            async fn wait_for_rising_edge(&mut self) -> Result<(), Self::Error> {
                unimplemented!();
            }

            async fn wait_for_falling_edge(&mut self) -> Result<(), Self::Error> {
                unimplemented!();
            }

            async fn wait_for_any_edge(&mut self) -> Result<(), Self::Error> {
                unimplemented!();
            }
        }
    };
}

pub mod input {
    use crate::peripheral::Peripheral;

    pub const SCHMITT_TRIGGER_CONFIGURABLE: bool = false;

    pub trait InputPin {}

    pub fn new(
        _pin: impl Peripheral<P: InputPin> + 'static,
        _pull: ariel_os_embassy_common::gpio::Pull,
        _schmitt_trigger: bool,
    ) -> Result<Input<'static>, ariel_os_embassy_common::gpio::input::Error> {
        unimplemented!();
    }

    #[cfg(feature = "external-interrupts")]
    pub fn new_int_enabled(
        _pin: impl Peripheral<P: InputPin> + 'static,
        _pull: ariel_os_embassy_common::gpio::Pull,
        _schmitt_trigger: bool,
    ) -> Result<IntEnabledInput<'static>, ariel_os_embassy_common::gpio::input::Error> {
        unimplemented!();
    }

    define_input_like!(Input);
    #[cfg(feature = "external-interrupts")]
    define_input_like!(IntEnabledInput);

    pub enum Level {
        Low,
        High,
    }

    ariel_os_embassy_common::define_into_level!();
}

pub mod output {
    use embedded_hal::digital::StatefulOutputPin;

    use crate::peripheral::Peripheral;

    pub const DRIVE_STRENGTH_CONFIGURABLE: bool = false;
    pub const SPEED_CONFIGURABLE: bool = false;

    pub trait OutputPin {}

    pub fn new(
        _pin: impl Peripheral<P: OutputPin> + 'static,
        _initial_level: ariel_os_embassy_common::gpio::Level,
        _drive_strength: super::DriveStrength,
        _speed: super::Speed,
    ) -> Output<'static> {
        unimplemented!();
    }

    pub struct Output<'d> {
        _marker: core::marker::PhantomData<&'d ()>,
    }

    impl embedded_hal::digital::ErrorType for Output<'_> {
        type Error = core::convert::Infallible;
    }

    impl embedded_hal::digital::OutputPin for Output<'_> {
        fn set_low(&mut self) -> Result<(), Self::Error> {
            unimplemented!();
        }

        fn set_high(&mut self) -> Result<(), Self::Error> {
            unimplemented!();
        }
    }

    impl StatefulOutputPin for Output<'_> {
        fn is_set_high(&mut self) -> Result<bool, Self::Error> {
            unimplemented!();
        }

        fn is_set_low(&mut self) -> Result<bool, Self::Error> {
            unimplemented!();
        }
    }
}

/// Actual type is HAL-specific.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum DriveStrength {
    #[doc(hidden)]
    Hidden,
}

impl ariel_os_embassy_common::gpio::FromDriveStrength for DriveStrength {
    fn from(_drive_strength: ariel_os_embassy_common::gpio::DriveStrength<Self>) -> Self {
        unimplemented!();
    }
}

/// Actual type is HAL-specific.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Speed {
    #[doc(hidden)]
    Hidden,
}

impl ariel_os_embassy_common::gpio::FromSpeed for Speed {
    fn from(_speed: ariel_os_embassy_common::gpio::Speed<Self>) -> Self {
        unimplemented!();
    }
}
