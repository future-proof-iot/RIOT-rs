pub mod output {
    use embassy_nrf::gpio::{Level, OutputDrive};

    use crate::{
        arch::peripheral::Peripheral,
        gpio::{FromDriveStrength, PinState},
    };

    pub(crate) use embassy_nrf::gpio::{Output, Pin};

    pub(crate) fn new(
        pin: impl Peripheral<P: Pin> + 'static,
        initial_state: PinState,
        drive_strength: DriveStrength,
    ) -> Output<'static> {
        let initial_state: bool = initial_state.into();
        let initial_state = Level::from(initial_state);
        // TODO: allow to set this as a setter (does not seem possible on nRF, but is on ESP)
        // TODO: this also depends on the open-drain configuration
        let output_drive = match drive_strength {
            DriveStrength::Standard => OutputDrive::Standard,
            DriveStrength::High => OutputDrive::HighDrive,
        };
        Output::new(pin, initial_state, output_drive)
    }

    #[derive(Copy, Clone, PartialEq, Eq)]
    pub enum DriveStrength {
        Standard,
        High, // Around 10 mA
    }

    impl Default for DriveStrength {
        fn default() -> Self {
            Self::Standard
        }
    }

    impl FromDriveStrength for DriveStrength {
        fn from(drive_strength: crate::gpio::DriveStrength) -> Self {
            use crate::gpio::DriveStrength::*;

            // ESPs are able to output up to 40 mA, so we somewhat normalize this.
            match drive_strength {
                Arch(drive_strength) => drive_strength,
                Lowest => DriveStrength::Standard,
                Standard => DriveStrength::default(),
                Medium => DriveStrength::Standard,
                High => DriveStrength::High,
                Highest => DriveStrength::High,
            }
        }
    }
}
