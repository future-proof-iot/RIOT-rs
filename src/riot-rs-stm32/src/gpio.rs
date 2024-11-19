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
        let pull = from_pull(pull);
        let mut pin = pin.into_ref();
        let ch = crate::extint_registry::EXTINT_REGISTRY.get_interrupt_channel_for_pin(&mut pin)?;
        let pin = pin.into_ref().map_into();
        Ok(IntEnabledInput::new(pin, ch, pull))
    }

    riot_rs_embassy_common::define_from_pull!();
    riot_rs_embassy_common::define_into_level!();
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

    #[derive(Copy, Clone, PartialEq, Eq)]
    pub enum DriveStrength {
        UnsupportedByHardware,
    }

    impl Default for DriveStrength {
        fn default() -> Self {
            Self::UnsupportedByHardware
        }
    }

    impl FromDriveStrength for DriveStrength {
        fn from(drive_strength: riot_rs_embassy_common::gpio::DriveStrength<Self>) -> Self {
            use riot_rs_embassy_common::gpio::DriveStrength::*;

            match drive_strength {
                Hal(drive_strength) => drive_strength,
                Lowest => DriveStrength::UnsupportedByHardware,
                Standard => DriveStrength::default(),
                Medium => DriveStrength::UnsupportedByHardware,
                High => DriveStrength::UnsupportedByHardware,
                Highest => DriveStrength::UnsupportedByHardware,
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
                Hal(speed) => speed,
                Low => Speed::Low,
                Medium => Speed::Medium,
                High => Speed::High,
                VeryHigh => Speed::VeryHigh,
            }
        }
    }
}
