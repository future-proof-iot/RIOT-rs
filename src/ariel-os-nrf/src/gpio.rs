pub mod input {
    use embassy_nrf::{
        gpio::{Level, Pull},
        Peripheral,
    };

    pub use embassy_nrf::gpio::{Input, Pin as InputPin};

    // Re-export `Input` as `IntEnabledInput` as they are interrupt-enabled.
    #[cfg(feature = "external-interrupts")]
    pub use embassy_nrf::gpio::Input as IntEnabledInput;

    pub const SCHMITT_TRIGGER_CONFIGURABLE: bool = false;

    pub fn new(
        pin: impl Peripheral<P: InputPin> + 'static,
        pull: ariel_os_embassy_common::gpio::Pull,
        _schmitt_trigger: bool, // Not supported by hardware
    ) -> Result<Input<'static>, ariel_os_embassy_common::gpio::input::Error> {
        let pull = from_pull(pull);
        Ok(Input::new(pin, pull))
    }

    #[cfg(feature = "external-interrupts")]
    pub fn new_int_enabled(
        pin: impl Peripheral<P: InputPin> + 'static,
        pull: ariel_os_embassy_common::gpio::Pull,
        _schmitt_trigger: bool, // Not supported by hardware
    ) -> Result<IntEnabledInput<'static>, ariel_os_embassy_common::gpio::input::Error> {
        let pull = from_pull(pull);
        let mut pin = pin.into_ref();
        crate::extint_registry::EXTINT_REGISTRY.use_interrupt_for_pin(&mut pin)?;
        Ok(Input::new(pin, pull))
    }

    ariel_os_embassy_common::define_from_pull!();
    ariel_os_embassy_common::define_into_level!();
}

pub mod output {
    use ariel_os_embassy_common::gpio::{FromDriveStrength, FromSpeed};
    use embassy_nrf::{
        gpio::{Level, OutputDrive},
        Peripheral,
    };

    pub use embassy_nrf::gpio::{Output, Pin as OutputPin};

    pub const DRIVE_STRENGTH_CONFIGURABLE: bool = true;
    pub const SPEED_CONFIGURABLE: bool = false;

    pub fn new(
        pin: impl Peripheral<P: OutputPin> + 'static,
        initial_level: ariel_os_embassy_common::gpio::Level,
        drive_strength: DriveStrength,
        _speed: Speed, // Not supported by hardware
    ) -> Output<'static> {
        let output_drive = match drive_strength {
            DriveStrength::Standard => OutputDrive::Standard,
            DriveStrength::High => OutputDrive::HighDrive,
        };
        let initial_level = match initial_level {
            ariel_os_embassy_common::gpio::Level::Low => Level::Low,
            ariel_os_embassy_common::gpio::Level::High => Level::High,
        };
        Output::new(pin, initial_level, output_drive)
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
        fn from(drive_strength: ariel_os_embassy_common::gpio::DriveStrength<Self>) -> Self {
            use ariel_os_embassy_common::gpio::DriveStrength::*;

            // ESPs are able to output up to 40 mA, so we somewhat normalize this.
            match drive_strength {
                Hal(drive_strength) => drive_strength,
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
        UnsupportedByHardware,
    }

    impl FromSpeed for Speed {
        fn from(_speed: ariel_os_embassy_common::gpio::Speed<Self>) -> Self {
            Self::UnsupportedByHardware
        }
    }
}
