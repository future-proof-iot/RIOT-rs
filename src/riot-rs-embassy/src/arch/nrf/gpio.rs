pub mod input {
    use embassy_nrf::gpio::{Level, Pull};

    use crate::{arch::peripheral::Peripheral, extint_registry::EXTINT_REGISTRY, gpio};

    pub(crate) use embassy_nrf::gpio::{Input, Pin};

    pub(crate) const SCHMITT_TRIGGER_AVAILABLE: bool = false;

    pub(crate) fn new(
        pin: impl Peripheral<P: Pin> + 'static,
        int_enabled: bool,
        pull: crate::gpio::Pull,
        _schmitt_trigger: bool, // Not supported by this architecture
    ) -> Result<Input<'static>, gpio::input::Error> {
        let pull = Pull::from(pull);

        let pin = if int_enabled {
            EXTINT_REGISTRY.use_interrupt_for_pin(pin)?
        } else {
            pin
        };

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
    use embassy_nrf::gpio::{Level, OutputDrive, Pull};

    use crate::{
        arch::peripheral::Peripheral,
        gpio::{FromDriveStrength, FromSpeed, PinState},
    };

    pub(crate) use embassy_nrf::gpio::{Flex as OpenDrainOutput, Output, Pin};

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
        let output_drive = match drive_strength {
            DriveStrength::Standard => OutputDrive::Standard,
            DriveStrength::High => OutputDrive::HighDrive,
        };
        Output::new(pin, initial_state, output_drive)
    }

    pub(crate) fn new_open_drain(
        pin: impl Peripheral<P: Pin> + 'static,
        initial_state: PinState,
        drive_strength: DriveStrength,
        pull: crate::gpio::Pull,
        _speed: Speed, // Not supported by this architecture
    ) -> OpenDrainOutput<'static> {
        // TODO: maybe factor this out with `new()`
        let initial_state: bool = initial_state.into();
        let initial_state = Level::from(initial_state);
        let output_drive = match drive_strength {
            DriveStrength::Standard => OutputDrive::Standard0Disconnect1,
            DriveStrength::High => OutputDrive::HighDrive0Disconnect1,
        };
        let pull = Pull::from(pull);

        let mut output = OpenDrainOutput::new(pin);
        // Initial state must be set before the pin is set to output.
        output.set_level(initial_state);
        output.set_as_input_output(pull, output_drive);
        output
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
