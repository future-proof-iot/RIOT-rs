pub mod input {
    use embassy_stm32::gpio::{Level, Pull};

    use crate::{arch::peripheral::Peripheral, extint_registry::EXTINT_REGISTRY, gpio};

    pub(crate) use embassy_stm32::{
        exti::ExtiInput as IntEnabledInput,
        gpio::{Input, Pin as InputPin},
    };

    pub(crate) const SCHMITT_TRIGGER_AVAILABLE: bool = false;

    pub(crate) fn new(
        pin: impl Peripheral<P: InputPin> + 'static,
        pull: crate::gpio::Pull,
        _schmitt_trigger: bool, // Not supported by this architecture
    ) -> Result<Input<'static>, gpio::input::Error> {
        let pull = Pull::from(pull);
        Ok(Input::new(pin, pull))
    }

    pub(crate) fn new_int_enabled<P: Peripheral<P = T> + 'static, T: InputPin>(
        pin: P,
        pull: crate::gpio::Pull,
        _schmitt_trigger: bool, // Not supported by this architecture
    ) -> Result<IntEnabledInput<'static>, gpio::input::Error> {
        let pull = Pull::from(pull);
        let mut pin = pin.into_ref();
        let ch = EXTINT_REGISTRY.get_interrupt_channel_for_pin(&mut pin)?;
        let pin = pin.into_ref().map_into();
        Ok(IntEnabledInput::new(pin, ch, pull))
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
    use embassy_stm32::gpio::{Level, Speed as StmSpeed};

    use crate::{
        arch::peripheral::Peripheral,
        gpio::{FromDriveStrength, FromSpeed},
    };

    pub(crate) use embassy_stm32::gpio::{Output, Pin as OutputPin};

    pub(crate) const DRIVE_STRENGTH_AVAILABLE: bool = false;
    pub(crate) const SPEED_AVAILABLE: bool = true;

    pub(crate) fn new(
        pin: impl Peripheral<P: OutputPin> + 'static,
        initial_level: crate::gpio::Level,
        _drive_strength: DriveStrength, // Not supported by this architecture
        speed: Speed,
    ) -> Output<'static> {
        Output::new(pin, initial_level.into(), speed.into())
    }

    crate::gpio::impl_from_level!(Level);

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
        fn from(drive_strength: crate::gpio::DriveStrength) -> Self {
            use crate::gpio::DriveStrength::*;

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
        fn from(speed: crate::gpio::Speed) -> Self {
            use crate::gpio::Speed::*;

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
