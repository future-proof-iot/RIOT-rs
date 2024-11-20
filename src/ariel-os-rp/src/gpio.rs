//! Provides GPIO access.

pub mod input {
    //! Input-specific types.

    use embassy_rp::{
        gpio::{Level, Pull},
        Peripheral,
    };

    #[cfg(feature = "external-interrupts")]
    use riot_rs_embassy_common::gpio::input::InterruptError;

    #[doc(hidden)]
    pub use embassy_rp::gpio::{Input, Pin as InputPin};

    // Re-export `Input` as `IntEnabledInput` as they are interrupt-enabled.
    #[cfg(feature = "external-interrupts")]
    #[doc(hidden)]
    pub use embassy_rp::gpio::Input as IntEnabledInput;

    /// Whether inputs support configuring whether a Schmitt trigger is enabled.
    pub const SCHMITT_TRIGGER_CONFIGURABLE: bool = true;

    #[doc(hidden)]
    pub fn new(
        pin: impl Peripheral<P: InputPin> + 'static,
        pull: riot_rs_embassy_common::gpio::Pull,
        schmitt_trigger: bool,
    ) -> Result<Input<'static>, core::convert::Infallible> {
        let pull = from_pull(pull);

        let mut input = Input::new(pin, pull);
        input.set_schmitt(schmitt_trigger);

        Ok(input)
    }

    #[cfg(feature = "external-interrupts")]
    #[doc(hidden)]
    pub fn new_int_enabled(
        pin: impl Peripheral<P: InputPin> + 'static,
        pull: riot_rs_embassy_common::gpio::Pull,
        schmitt_trigger: bool,
    ) -> Result<IntEnabledInput<'static>, InterruptError> {
        // This HAL does not require special treatment of external interrupts.
        match new(pin, pull, schmitt_trigger) {
            Ok(input) => Ok(input),
            Err(err) => match err {
                // Compile-time check that this never happens as the Result is Infallible.
            },
        }
    }

    riot_rs_embassy_common::define_from_pull!();
    riot_rs_embassy_common::define_into_level!();
}

pub mod output {
    //! Output-specific types.

    use embassy_rp::{
        gpio::{Drive, Level, SlewRate},
        Peripheral,
    };
    use riot_rs_embassy_common::gpio::{FromDriveStrength, FromSpeed};

    #[doc(hidden)]
    pub use embassy_rp::gpio::{Output, Pin as OutputPin};

    /// Whether outputs support configuring their drive strength.
    pub const DRIVE_STRENGTH_CONFIGURABLE: bool = true;
    /// Whether outputs support configuring their speed/slew rate.
    pub const SPEED_CONFIGURABLE: bool = true;

    #[doc(hidden)]
    pub fn new(
        pin: impl Peripheral<P: OutputPin> + 'static,
        initial_level: riot_rs_embassy_common::gpio::Level,
        drive_strength: DriveStrength,
        speed: Speed,
    ) -> Output<'static> {
        let initial_level = match initial_level {
            riot_rs_embassy_common::gpio::Level::Low => Level::Low,
            riot_rs_embassy_common::gpio::Level::High => Level::High,
        };
        let mut output = Output::new(pin, initial_level);
        output.set_drive_strength(drive_strength.into());
        output.set_slew_rate(speed.into());
        output
    }

    /// Available drive strength settings.
    // We provide our own type because the upstream type is not `Copy` and has no `Default` impl.
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub enum DriveStrength {
        /// 2 mA.
        _2mA,
        /// 4 mA.
        _4mA,
        /// 8 mA.
        _8mA,
        /// 12 mA.
        _12mA,
    }

    impl Default for DriveStrength {
        fn default() -> Self {
            // Reset value
            Self::_4mA
        }
    }

    impl From<DriveStrength> for Drive {
        fn from(drive_strength: DriveStrength) -> Self {
            match drive_strength {
                DriveStrength::_2mA => Self::_2mA,
                DriveStrength::_4mA => Self::_4mA,
                DriveStrength::_8mA => Self::_8mA,
                DriveStrength::_12mA => Self::_12mA,
            }
        }
    }

    impl FromDriveStrength for DriveStrength {
        fn from(drive_strength: riot_rs_embassy_common::gpio::DriveStrength<Self>) -> Self {
            use riot_rs_embassy_common::gpio::DriveStrength::*;

            // ESPs are able to output up to 40 mA, so we somewhat normalize this.
            match drive_strength {
                Hal(drive_strength) => drive_strength,
                Lowest => DriveStrength::_2mA,
                Standard => DriveStrength::default(),
                Medium => DriveStrength::_8mA,
                High => DriveStrength::_12mA,
                Highest => DriveStrength::_12mA,
            }
        }
    }

    /// Available output speed/slew rate settings.
    // These values do not seem to be quantitatively defined on the RP2040.
    // We provide our own type because the `SlewRate` upstream type is not `Copy` and has no
    // `Default` impl.
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub enum Speed {
        /// Low.
        Low,
        /// High.
        High,
    }

    impl Default for Speed {
        fn default() -> Self {
            // Reset value
            Self::Low
        }
    }

    impl From<Speed> for SlewRate {
        fn from(speed: Speed) -> Self {
            match speed {
                Speed::Low => SlewRate::Slow,
                Speed::High => SlewRate::Fast,
            }
        }
    }

    impl FromSpeed for Speed {
        fn from(speed: riot_rs_embassy_common::gpio::Speed<Self>) -> Self {
            use riot_rs_embassy_common::gpio::Speed::*;

            match speed {
                Hal(speed) => speed,
                Low => Speed::Low,
                Medium => Speed::Low,
                High => Speed::High,
                VeryHigh => Speed::High,
            }
        }
    }
}
