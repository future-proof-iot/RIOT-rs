pub mod input {
    use esp_hal::gpio::{Level, Pull};

    use crate::{arch::peripheral::Peripheral, gpio};

    pub(crate) use esp_hal::gpio::{AnyInput as Input, Pin};

    pub(crate) const SCHMITT_TRIGGER_AVAILABLE: bool = false;

    pub(crate) fn new(
        pin: impl Peripheral<P: Pin> + 'static,
        int_enabled: bool,
        pull: crate::gpio::Pull,
        _schmitt_trigger: bool, // Not supported by this architecture
    ) -> Result<Input<'static>, gpio::input::Error> {
        let pull = Pull::from(pull);

        Ok(Input::new(pin, pull))
    }

    impl From<crate::gpio::Pull> for Pull {
        fn from(pull: crate::gpio::Pull) -> Self {
            match pull {
                crate::gpio::Pull::None => Pull::None,
                crate::gpio::Pull::Up => Pull::Up,
                crate::gpio::Pull::Down => Pull::Down,
            }
        }
    }

    impl From<Level> for crate::gpio::Level {
        fn from(level: Level) -> Self {
            match level {
                Level::Low => crate::gpio::Level::Low,
                Level::High => crate::gpio::Level::High,
            }
        }
    }
}

pub mod output {
    use esp_hal::gpio::Level;

    use crate::{
        arch::peripheral::Peripheral,
        gpio::{FromDriveStrength, FromSpeed, PinState},
    };

    pub(crate) use esp_hal::gpio::{AnyOutput as Output, Pin};

    pub(crate) const DRIVE_STRENGTH_AVAILABLE: bool = true;
    pub(crate) const SPEED_AVAILABLE: bool = false;

    pub(crate) fn new(
        pin: impl Peripheral<P: Pin> + 'static,
        initial_state: PinState,
        drive_strength: DriveStrength,
        _speed: Speed, // Not supported by this architecture
    ) -> Output<'static> {
        let initial_state: bool = initial_state.into();
        let initial_state = Level::from(initial_state);
        let mut output = Output::new(pin, initial_state);
        output.set_drive_strength(drive_strength.into());
        output
    }

    // We do not provide a `Default` impl as not all pins have the same reset value.
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub enum DriveStrength {
        _5mA,
        _10mA,
        _20mA,
        _40mA,
    }

    impl FromDriveStrength for DriveStrength {
        fn from(drive_strength: crate::gpio::DriveStrength) -> Self {
            use crate::gpio::DriveStrength::*;

            // ESPs are able to output up to 40 mA, so we somewhat normalize this.
            match drive_strength {
                Arch(drive_strength) => drive_strength,
                Lowest => DriveStrength::_5mA,
                Standard => DriveStrength::_10mA,
                Medium => DriveStrength::_10mA,
                High => DriveStrength::_20mA,
                Highest => DriveStrength::_40mA,
            }
        }
    }

    #[derive(Copy, Clone, PartialEq, Eq)]
    pub enum Speed {
        UnsupportedByArchitecture,
    }

    impl FromSpeed for Speed {
        fn from(_speed: crate::gpio::Speed) -> Self {
            Self::UnsupportedByArchitecture
        }
    }
}
