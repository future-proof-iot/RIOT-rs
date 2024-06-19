pub(crate) use embassy_rp::gpio::Pin;

pub mod output {
    use embassy_rp::gpio::{Drive, Level, Pin};

    use crate::gpio::{FromDriveStrength, PinState};

    pub(crate) use embassy_rp::gpio::Output;

    pub(crate) fn new(
        pin: impl Pin,
        initial_state: PinState,
        drive_strength: DriveStrength,
    ) -> Output<'static> {
        let initial_state: bool = initial_state.into();
        let initial_state = Level::from(initial_state);
        // TODO: allow to set this as a setter (does not seem possible on nRF, but is on ESP)
        let mut output = Output::new(pin, initial_state);
        output.set_drive_strength(drive_strength.into());
        output
    }

    // We provide our own type because the upstream Drive is not `Copy` and has no `Default` impl.
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub enum DriveStrength {
        _2mA,
        _4mA,
        _8mA,
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
        fn from(drive_strength: crate::gpio::DriveStrength) -> Self {
            use crate::gpio::DriveStrength::*;

            // ESPs are able to output up to 40 mA, so we somewhat normalize this.
            match drive_strength {
                Arch(drive_strength) => drive_strength,
                Lowest => DriveStrength::_2mA,
                Standard => DriveStrength::default(),
                Medium => DriveStrength::_8mA,
                High => DriveStrength::_12mA,
                Highest => DriveStrength::_12mA,
            }
        }
    }
}
