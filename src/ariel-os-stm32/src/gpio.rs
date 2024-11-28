//! Provides GPIO access.

pub mod input {
    //! Input-specific types.

    use embassy_stm32::{
        gpio::{Level, Pull},
        Peripheral,
    };

    #[doc(hidden)]
    pub use embassy_stm32::gpio::{Input, Pin as InputPin};

    #[cfg(feature = "external-interrupts")]
    #[doc(hidden)]
    pub use embassy_stm32::exti::ExtiInput as IntEnabledInput;

    /// Whether inputs support configuring whether a Schmitt trigger is enabled.
    pub const SCHMITT_TRIGGER_CONFIGURABLE: bool = false;

    #[doc(hidden)]
    pub fn new(
        pin: impl Peripheral<P: InputPin> + 'static,
        pull: ariel_os_embassy_common::gpio::Pull,
        _schmitt_trigger: bool, // Not supported by this hardware
    ) -> Result<Input<'static>, ariel_os_embassy_common::gpio::input::Error> {
        let pull = from_pull(pull);
        Ok(Input::new(pin, pull))
    }

    #[cfg(feature = "external-interrupts")]
    #[doc(hidden)]
    pub fn new_int_enabled<P: Peripheral<P = T> + 'static, T: InputPin>(
        pin: P,
        pull: ariel_os_embassy_common::gpio::Pull,
        _schmitt_trigger: bool, // Not supported by this hardware
    ) -> Result<IntEnabledInput<'static>, ariel_os_embassy_common::gpio::input::Error> {
        let pull = from_pull(pull);
        let mut pin = pin.into_ref();
        let ch = crate::extint_registry::EXTINT_REGISTRY.get_interrupt_channel_for_pin(&mut pin)?;
        let pin = pin.into_ref().map_into();
        Ok(IntEnabledInput::new(pin, ch, pull))
    }

    ariel_os_embassy_common::define_from_pull!();
    ariel_os_embassy_common::define_into_level!();
}

pub mod output {
    //! Output-specific types.

    use ariel_os_embassy_common::gpio::FromSpeed;
    use embassy_stm32::{
        gpio::{Level, Speed as StmSpeed},
        Peripheral,
    };

    pub use ariel_os_embassy_common::gpio::UnsupportedDriveStrength as DriveStrength;

    #[doc(hidden)]
    pub use embassy_stm32::gpio::{Output, Pin as OutputPin};

    /// Whether outputs support configuring their drive strength.
    pub const DRIVE_STRENGTH_CONFIGURABLE: bool = false;
    /// Whether outputs support configuring their speed/slew rate.
    pub const SPEED_CONFIGURABLE: bool = true;

    #[doc(hidden)]
    pub fn new(
        pin: impl Peripheral<P: OutputPin> + 'static,
        initial_level: ariel_os_embassy_common::gpio::Level,
        _drive_strength: DriveStrength, // Not supported by hardware
        speed: Speed,
    ) -> Output<'static> {
        let initial_level = match initial_level {
            ariel_os_embassy_common::gpio::Level::Low => Level::Low,
            ariel_os_embassy_common::gpio::Level::High => Level::High,
        };
        Output::new(pin, initial_level, speed.into())
    }

    /// Available output speed/slew rate settings.
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub enum Speed {
        /// Low.
        Low,
        /// Medium.
        Medium,
        /// High.
        High,
        /// Very high.
        VeryHigh,
    }

    impl From<Speed> for StmSpeed {
        fn from(speed: Speed) -> Self {
            match speed {
                Speed::Low => StmSpeed::Low,
                Speed::Medium => StmSpeed::Medium,
                Speed::High => StmSpeed::High,
                Speed::VeryHigh => StmSpeed::VeryHigh,
            }
        }
    }

    impl FromSpeed for Speed {
        fn from(speed: ariel_os_embassy_common::gpio::Speed<Self>) -> Self {
            use ariel_os_embassy_common::gpio::Speed::*;

            match speed {
                Hal(speed) => speed,
                Low => Speed::Low,
                Medium => Speed::Medium,
                High => Speed::High,
                VeryHigh => Speed::VeryHigh,
            }
        }
    }
}
