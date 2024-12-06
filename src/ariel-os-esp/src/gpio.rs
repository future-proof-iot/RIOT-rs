//! Provides GPIO access.

pub mod input {
    //! Input-specific types.

    use esp_hal::{
        gpio::{Level, Pull},
        peripheral::Peripheral,
    };

    #[doc(hidden)]
    pub use esp_hal::gpio::{Input, InputPin};

    #[cfg(feature = "external-interrupts")]
    use ariel_os_embassy_common::gpio::input::InterruptError;

    // Re-export `Input` as `IntEnabledInput` as they are interrupt-enabled.
    #[cfg(feature = "external-interrupts")]
    #[doc(hidden)]
    pub use esp_hal::gpio::Input as IntEnabledInput;

    /// Whether inputs support configuring whether a Schmitt trigger is enabled.
    pub const SCHMITT_TRIGGER_CONFIGURABLE: bool = false;

    #[doc(hidden)]
    pub fn new(
        pin: impl Peripheral<P: InputPin> + 'static,
        pull: ariel_os_embassy_common::gpio::Pull,
        _schmitt_trigger: bool, // Not supported by hardware
    ) -> Result<Input<'static>, core::convert::Infallible> {
        let pull = from_pull(pull);

        Ok(Input::new(pin, pull))
    }

    #[cfg(feature = "external-interrupts")]
    #[doc(hidden)]
    pub fn new_int_enabled(
        pin: impl Peripheral<P: InputPin> + 'static,
        pull: ariel_os_embassy_common::gpio::Pull,
        _schmitt_trigger: bool, // Not supported by hardware
    ) -> Result<IntEnabledInput<'static>, InterruptError> {
        match new(pin, pull, _schmitt_trigger) {
            Ok(input) => Ok(input),
            Err(err) => match err {
                // Compile-time check that this never happens as the Result is Infallible.
            },
        }
    }

    ariel_os_embassy_common::define_from_pull!();
    ariel_os_embassy_common::define_into_level!();
}

pub mod output {
    //! Output-specific types.

    use ariel_os_embassy_common::gpio::FromDriveStrength;
    use esp_hal::{gpio::Level, peripheral::Peripheral};

    pub use ariel_os_embassy_common::gpio::UnsupportedSpeed as Speed;

    #[doc(hidden)]
    pub use esp_hal::gpio::{Output, OutputPin};

    /// Whether outputs support configuring their drive strength.
    pub const DRIVE_STRENGTH_CONFIGURABLE: bool = true;
    /// Whether outputs support configuring their speed/slew rate.
    pub const SPEED_CONFIGURABLE: bool = false;

    #[doc(hidden)]
    pub fn new(
        pin: impl Peripheral<P: OutputPin> + 'static,
        initial_level: ariel_os_embassy_common::gpio::Level,
        drive_strength: DriveStrength,
        _speed: Speed, // Not supported by hardware
    ) -> Output<'static> {
        let initial_level = match initial_level {
            ariel_os_embassy_common::gpio::Level::Low => Level::Low,
            ariel_os_embassy_common::gpio::Level::High => Level::High,
        };
        let mut output = Output::new(pin, initial_level);
        output.set_drive_strength(drive_strength.into());
        output
    }

    /// Available drive strength settings.
    // We do not provide a `Default` impl as not all pins have the same reset value.
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub enum DriveStrength {
        /// 5 mA.
        _5mA,
        /// 10 mA.
        _10mA,
        /// 20 mA.
        _20mA,
        /// 40 mA.
        _40mA,
    }

    impl From<DriveStrength> for esp_hal::gpio::DriveStrength {
        fn from(drive_strength: DriveStrength) -> Self {
            match drive_strength {
                DriveStrength::_5mA => esp_hal::gpio::DriveStrength::I5mA,
                DriveStrength::_10mA => esp_hal::gpio::DriveStrength::I10mA,
                DriveStrength::_20mA => esp_hal::gpio::DriveStrength::I20mA,
                DriveStrength::_40mA => esp_hal::gpio::DriveStrength::I40mA,
            }
        }
    }

    impl FromDriveStrength for DriveStrength {
        fn from(drive_strength: ariel_os_embassy_common::gpio::DriveStrength<Self>) -> Self {
            use ariel_os_embassy_common::gpio::DriveStrength::*;

            // ESPs are able to output up to 40 mA, so we somewhat normalize this.
            match drive_strength {
                Hal(drive_strength) => drive_strength,
                Lowest => DriveStrength::_5mA,
                Standard => DriveStrength::_10mA,
                Medium => DriveStrength::_10mA,
                High => DriveStrength::_20mA,
                Highest => DriveStrength::_40mA,
            }
        }
    }
}
