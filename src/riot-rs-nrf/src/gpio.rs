pub trait IntoLevel {
    fn into(level: Self) -> riot_rs_embassy_common::gpio::Level;
}

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
        pull: riot_rs_embassy_common::gpio::Pull,
        _schmitt_trigger: bool, // Not supported by this architecture
    ) -> Result<Input<'static>, riot_rs_embassy_common::gpio::input::Error> {
        let pull = from_pull(pull);
        Ok(Input::new(pin, pull))
    }

    #[cfg(feature = "external-interrupts")]
    pub fn new_int_enabled(
        pin: impl Peripheral<P: InputPin> + 'static,
        pull: riot_rs_embassy_common::gpio::Pull,
        _schmitt_trigger: bool, // Not supported by this architecture
    ) -> Result<IntEnabledInput<'static>, riot_rs_embassy_common::gpio::input::Error> {
        let pull = from_pull(pull);
        let mut pin = pin.into_ref();
        crate::extint_registry::EXTINT_REGISTRY.use_interrupt_for_pin(&mut pin)?;
        Ok(Input::new(pin, pull))
    }

    fn from_pull(pull: riot_rs_embassy_common::gpio::Pull) -> Pull {
        match pull {
            riot_rs_embassy_common::gpio::Pull::None => Pull::None,
            riot_rs_embassy_common::gpio::Pull::Up => Pull::Up,
            riot_rs_embassy_common::gpio::Pull::Down => Pull::Down,
        }
    }

    impl crate::gpio::IntoLevel for Level {
        fn into(level: Self) -> riot_rs_embassy_common::gpio::Level {
            match level {
                Level::Low => riot_rs_embassy_common::gpio::Level::Low,
                Level::High => riot_rs_embassy_common::gpio::Level::High,
            }
        }
    }

    // impl From<crate::gpio::Pull> for Pull {
    //     fn from(pull: crate::gpio::Pull) -> Self {
    //         match pull {
    //             crate::gpio::Pull::None => Pull::None,
    //             crate::gpio::Pull::Up => Pull::Up,
    //             crate::gpio::Pull::Down => Pull::Down,
    //         }
    //     }
    // }
    //
    // impl From<Level> for crate::gpio::Level {
    //     fn from(level: Level) -> Self {
    //         match level {
    //             Level::Low => crate::gpio::Level::Low,
    //             Level::High => crate::gpio::Level::High,
    //         }
    //     }
    // }
}

pub mod output {
    use embassy_nrf::{
        gpio::{Level, OutputDrive},
        Peripheral,
    };
    use riot_rs_embassy_common::gpio::{FromDriveStrength, FromSpeed};

    pub use embassy_nrf::gpio::{Output, Pin as OutputPin};

    pub const DRIVE_STRENGTH_CONFIGURABLE: bool = true;
    pub const SPEED_CONFIGURABLE: bool = false;

    pub fn new(
        pin: impl Peripheral<P: OutputPin> + 'static,
        initial_level: riot_rs_embassy_common::gpio::Level,
        drive_strength: DriveStrength,
        _speed: Speed, // Not supported by this architecture
    ) -> Output<'static> {
        let output_drive = match drive_strength {
            DriveStrength::Standard => OutputDrive::Standard,
            DriveStrength::High => OutputDrive::HighDrive,
        };
        let initial_level = match initial_level {
            riot_rs_embassy_common::gpio::Level::Low => Level::Low,
            riot_rs_embassy_common::gpio::Level::High => Level::High,
        };
        Output::new(pin, initial_level, output_drive)
    }

    // crate::gpio::impl_from_level!(Level);

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
        fn from(drive_strength: riot_rs_embassy_common::gpio::DriveStrength<Self>) -> Self {
            use riot_rs_embassy_common::gpio::DriveStrength::*;

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
        fn from(_speed: riot_rs_embassy_common::gpio::Speed<Self>) -> Self {
            Self::UnsupportedByArchitecture
        }
    }
}
