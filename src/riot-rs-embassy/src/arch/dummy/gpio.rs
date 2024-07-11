macro_rules! define_input_like {
    ($type:ident) => {
        pub struct $type<'d> {
            _marker: core::marker::PhantomData<&'d ()>,
        }

        impl $type<'_> {
            pub fn is_high(&self) -> bool {
                unimplemented!();
            }

            pub fn is_low(&self) -> bool {
                unimplemented!();
            }

            pub fn get_level(&self) -> crate::gpio::Level {
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
    use crate::{arch::peripheral::Peripheral, gpio};

    pub(crate) const SCHMITT_TRIGGER_CONFIGURABLE: bool = false;

    pub trait InputPin {}

    pub(crate) fn new(
        _pin: impl Peripheral<P: InputPin> + 'static,
        _pull: crate::gpio::Pull,
        _schmitt_trigger: bool,
    ) -> Result<Input<'static>, gpio::input::Error> {
        unimplemented!();
    }

    pub(crate) fn new_int_enabled(
        _pin: impl Peripheral<P: InputPin> + 'static,
        _pull: crate::gpio::Pull,
        _schmitt_trigger: bool,
    ) -> Result<IntEnabledInput<'static>, gpio::input::Error> {
        unimplemented!();
    }

    define_input_like!(Input);
    define_input_like!(IntEnabledInput);
}

pub mod output {
    use embedded_hal::digital::StatefulOutputPin;

    use crate::{
        arch::peripheral::Peripheral,
        gpio::{FromDriveStrength, FromSpeed},
    };

    pub(crate) const DRIVE_STRENGTH_CONFIGURABLE: bool = false;
    pub(crate) const SPEED_CONFIGURABLE: bool = false;

    pub trait OutputPin {}

    pub(crate) fn new(
        _pin: impl Peripheral<P: OutputPin> + 'static,
        _initial_level: crate::gpio::Level,
        _drive_strength: DriveStrength,
        _speed: Speed,
    ) -> Output<'static> {
        unimplemented!();
    }

    /// Actual type is architecture-specific.
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub enum DriveStrength {}

    impl FromDriveStrength for DriveStrength {
        fn from(_drive_strength: crate::gpio::DriveStrength) -> Self {
            unimplemented!();
        }
    }

    /// Actual type is architecture-specific.
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub enum Speed {}

    impl FromSpeed for Speed {
        fn from(_speed: crate::gpio::Speed) -> Self {
            unimplemented!();
        }
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
