pub trait IntoLevel {
    fn into(level: Self) -> riot_rs_embassy_common::gpio::Level;
}

pub mod input {
    use embassy_stm32::{
        gpio::{Level, Pull},
        Peripheral,
    };

    pub use embassy_stm32::gpio::{Input, Pin as InputPin};

    #[cfg(feature = "external-interrupts")]
    pub use embassy_stm32::exti::ExtiInput as IntEnabledInput;

    pub const SCHMITT_TRIGGER_CONFIGURABLE: bool = false;

    pub fn new(
        pin: impl Peripheral<P: InputPin> + 'static,
        pull: riot_rs_embassy_common::gpio::Pull,
        _schmitt_trigger: bool, // Not supported by this architecture
        // let pull = Pull::from(pull);
    ) -> Result<Input<'static>, riot_rs_embassy_common::gpio::input::Error> {
        let pull = from_pull(pull);
        Ok(Input::new(pin, pull))
    }

    #[cfg(feature = "external-interrupts")]
    pub fn new_int_enabled<P: Peripheral<P = T> + 'static, T: InputPin>(
        pin: P,
        pull: riot_rs_embassy_common::gpio::Pull,
        _schmitt_trigger: bool, // Not supported by this architecture
    ) -> Result<IntEnabledInput<'static>, riot_rs_embassy_common::gpio::input::Error> {
        // let pull = Pull::from(pull);
        let pull = from_pull(pull);
        let mut pin = pin.into_ref();
        let ch = crate::extint_registry::EXTINT_REGISTRY.get_interrupt_channel_for_pin(&mut pin)?;
        let pin = pin.into_ref().map_into();
        Ok(IntEnabledInput::new(pin, ch, pull))
    }

    impl crate::gpio::IntoLevel for Level {
        fn into(level: Self) -> riot_rs_embassy_common::gpio::Level {
            match level {
                Level::Low => riot_rs_embassy_common::gpio::Level::Low,
                Level::High => riot_rs_embassy_common::gpio::Level::High,
            }
        }
    }

    fn from_pull(pull: riot_rs_embassy_common::gpio::Pull) -> Pull {
        match pull {
            riot_rs_embassy_common::gpio::Pull::None => Pull::None,
            riot_rs_embassy_common::gpio::Pull::Up => Pull::Up,
            riot_rs_embassy_common::gpio::Pull::Down => Pull::Down,
        }
    }

    // impl From<riot_rs_embassy_common::gpio::Pull> for Pull {
    //     fn from(pull: riot_rs_embassy_common::gpio::Pull) -> Self {
    //         match pull {
    //             riot_rs_embassy_common::gpio::Pull::None => Pull::None,
    //             riot_rs_embassy_common::gpio::Pull::Up => Pull::Up,
    //             riot_rs_embassy_common::gpio::Pull::Down => Pull::Down,
    //         }
    //     }
    // }
    //
    // impl From<Level> for riot_rs_embassy_common::gpio::Level {
    //     fn from(level: Level) -> Self {
    //         match level {
    //             Level::Low => riot_rs_embassy_common::gpio::Level::Low,
    //             Level::High => riot_rs_embassy_common::gpio::Level::High,
    //         }
    //     }
    // }
}

pub mod output {
    use embassy_stm32::{
        gpio::{Level, Speed as StmSpeed},
        Peripheral,
    };

    use riot_rs_embassy_common::gpio::{FromDriveStrength, FromSpeed};

    pub use embassy_stm32::gpio::{Output, Pin as OutputPin};

    pub const DRIVE_STRENGTH_CONFIGURABLE: bool = false;
    pub const SPEED_CONFIGURABLE: bool = true;

    pub fn new(
        pin: impl Peripheral<P: OutputPin> + 'static,
        initial_level: riot_rs_embassy_common::gpio::Level,
        _drive_strength: DriveStrength, // Not supported by this architecture
        speed: Speed,
    ) -> Output<'static> {
        let initial_level = match initial_level {
            riot_rs_embassy_common::gpio::Level::Low => Level::Low,
            riot_rs_embassy_common::gpio::Level::High => Level::High,
        };
        Output::new(pin, initial_level, speed.into())
    }

    // riot_rs_embassy_common::impl_from_level!(Level);

    #[derive(Copy, Clone, PartialEq, Eq)]
    pub enum DriveStrength {
        UnsupportedByArchitecture,
    }

    impl Default for DriveStrength {
        fn default() -> Self {
            Self::UnsupportedByArchitecture
        }
    }

    impl FromDriveStrength for DriveStrength {
        fn from(drive_strength: riot_rs_embassy_common::gpio::DriveStrength<Self>) -> Self {
            use riot_rs_embassy_common::gpio::DriveStrength::*;

            match drive_strength {
                Arch(drive_strength) => drive_strength,
                Lowest => DriveStrength::UnsupportedByArchitecture,
                Standard => DriveStrength::default(),
                Medium => DriveStrength::UnsupportedByArchitecture,
                High => DriveStrength::UnsupportedByArchitecture,
                Highest => DriveStrength::UnsupportedByArchitecture,
            }
        }
    }

    #[derive(Copy, Clone, PartialEq, Eq)]
    pub enum Speed {
        Low,
        Medium,
        High,
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
        fn from(speed: riot_rs_embassy_common::gpio::Speed<Self>) -> Self {
            use riot_rs_embassy_common::gpio::Speed::*;

            match speed {
                Arch(speed) => speed,
                Low => Speed::Low,
                Medium => Speed::Medium,
                High => Speed::High,
                VeryHigh => Speed::VeryHigh,
            }
        }
    }
}
