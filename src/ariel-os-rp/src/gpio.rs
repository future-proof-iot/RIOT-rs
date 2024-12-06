//! Provides GPIO access.

pub mod input {
    //! Input-specific types.

    use embassy_rp::{
        gpio::{Level, Pull},
        Peripheral,
    };

    #[cfg(feature = "external-interrupts")]
    use ariel_os_embassy_common::gpio::input::InterruptError;

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
        pull: ariel_os_embassy_common::gpio::Pull,
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
        pull: ariel_os_embassy_common::gpio::Pull,
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

    ariel_os_embassy_common::define_from_pull!();
    ariel_os_embassy_common::define_into_level!();
}

pub mod output {
    //! Output-specific types.

    use embassy_rp::{gpio::Level, Peripheral};

    #[doc(hidden)]
    pub use embassy_rp::gpio::{Output, Pin as OutputPin};

    /// Whether outputs support configuring their drive strength.
    pub const DRIVE_STRENGTH_CONFIGURABLE: bool = true;
    /// Whether outputs support configuring their speed/slew rate.
    pub const SPEED_CONFIGURABLE: bool = true;

    #[doc(hidden)]
    pub fn new(
        pin: impl Peripheral<P: OutputPin> + 'static,
        initial_level: ariel_os_embassy_common::gpio::Level,
        drive_strength: super::DriveStrength,
        speed: super::Speed,
    ) -> Output<'static> {
        let initial_level = match initial_level {
            ariel_os_embassy_common::gpio::Level::Low => Level::Low,
            ariel_os_embassy_common::gpio::Level::High => Level::High,
        };
        let mut output = Output::new(pin, initial_level);
        output.set_drive_strength(drive_strength.into());
        output.set_slew_rate(speed.into());
        output
    }
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

impl From<DriveStrength> for embassy_rp::gpio::Drive {
    fn from(drive_strength: DriveStrength) -> Self {
        match drive_strength {
            DriveStrength::_2mA => Self::_2mA,
            DriveStrength::_4mA => Self::_4mA,
            DriveStrength::_8mA => Self::_8mA,
            DriveStrength::_12mA => Self::_12mA,
        }
    }
}

impl ariel_os_embassy_common::gpio::FromDriveStrength for DriveStrength {
    fn from(drive_strength: ariel_os_embassy_common::gpio::DriveStrength<Self>) -> Self {
        use ariel_os_embassy_common::gpio::DriveStrength::*;

        // ESPs are able to output up to 40 mA, so we somewhat normalize this.
        match drive_strength {
            Hal(drive_strength) => drive_strength,
            Lowest => Self::_2mA,
            Standard => Self::default(),
            Medium => Self::_8mA,
            High => Self::_12mA,
            Highest => Self::_12mA,
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

impl From<Speed> for embassy_rp::gpio::SlewRate {
    fn from(speed: Speed) -> Self {
        match speed {
            Speed::Low => Self::Slow,
            Speed::High => Self::Fast,
        }
    }
}

impl ariel_os_embassy_common::gpio::FromSpeed for Speed {
    fn from(speed: ariel_os_embassy_common::gpio::Speed<Self>) -> Self {
        use ariel_os_embassy_common::gpio::Speed::*;

        match speed {
            Hal(speed) => speed,
            Low => Self::Low,
            Medium => Self::Low,
            High => Self::High,
            VeryHigh => Self::High,
        }
    }
}
