use embedded_hal::digital::{OutputPin, StatefulOutputPin};

use crate::arch::{
    self,
    gpio::{
        input::{Input as ArchInput, Pin as ArchInputPin},
        output::{
            DriveStrength as ArchDriveStrength, Output as ArchOutput, Pin as ArchOutputPin,
            Speed as ArchSpeed,
        },
    },
    peripheral::Peripheral,
};

pub use embedded_hal::digital::PinState;

pub struct Input {
    input: ArchInput<'static>, // FIXME: is this ok to require a 'static pin?
}

// FIXME: impl Wait + same methods (/!\ STM32)
impl Input {
    pub fn new(pin: impl Peripheral<P: ArchInputPin> + 'static, pull: Pull) -> Self {
        Self::builder(pin, pull).build()
    }

    pub fn builder<P: Peripheral<P: ArchInputPin>>(pin: P, pull: Pull) -> InputBuilder<P> {
        InputBuilder {
            pin,
            pull,
            schmitt_trigger: false,
        }
    }

    pub fn is_high(&self) -> bool {
        self.input.is_high()
    }

    pub fn is_low(&self) -> bool {
        self.input.is_low()
    }

    pub fn get_level(&self) -> Level {
        self.input.get_level().into()
    }
}

// TODO: should we use PinState instead?
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Level {
    Low,
    High,
}

impl From<Level> for bool {
    fn from(level: Level) -> Self {
        match level {
            Level::Low => false,
            Level::High => true,
        }
    }
}

impl From<bool> for Level {
    fn from(boolean: bool) -> Self {
        match boolean {
            false => Level::Low,
            true => Level::High,
        }
    }
}

pub struct InputBuilder<P: Peripheral<P: ArchInputPin>> {
    pin: P,
    pull: Pull,
    schmitt_trigger: bool,
}

impl<P: Peripheral<P: ArchInputPin> + 'static> InputBuilder<P> {
    pub fn schmitt_trigger(self, enable: bool) -> Self {
        const {
            assert!(
                arch::gpio::input::SCHMITT_TRIGGER_AVAILABLE,
                "This architecture does not support enabling Schmitt triggers on GPIO inputs."
            );
        }

        Self {
            schmitt_trigger: enable,
            ..self
        }
    }

    // It is unclear whether `opt_*()` functions are actually useful, so we provide them but do not
    // commit to them being part of our API for now.
    // We may remove them in the future if we realize they are never useful.
    #[doc(hidden)]
    pub fn opt_schmitt_trigger(self, enable: bool) -> Self {
        if arch::gpio::input::SCHMITT_TRIGGER_AVAILABLE {
            // We cannot reuse the non-`opt_*()`, otherwise the const assert inside it would always
            // be triggered.
            Self {
                schmitt_trigger: enable,
                ..self
            }
        } else {
            self
        }
    }

    pub fn build(self) -> Input {
        let input = arch::gpio::input::new(self.pin, self.pull, self.schmitt_trigger);

        Input { input }
    }
}

// All the architectures we support have pull-up and pull-down resistors.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Pull {
    None,
    Up,
    Down,
}

pub struct Output {
    output: ArchOutput<'static>, // FIXME: is this ok to require a 'static pin?
}

impl Output {
    // TODO: is PinState appropriate if we turn this into a open-drain-capable output?
    pub fn new(pin: impl Peripheral<P: ArchOutputPin> + 'static, initial_state: PinState) -> Self {
        Self::builder(pin, initial_state).build()
    }

    pub fn builder<P: Peripheral<P: ArchOutputPin>>(
        pin: P,
        initial_state: PinState,
    ) -> OutputBuilder<P> {
        OutputBuilder {
            pin,
            initial_state,
            drive_strength: DriveStrength::default(),
            speed: Speed::default(),
        }
    }

    pub fn set_low(&mut self) {
        // All architectures are infallible.
        let _ = <Self as OutputPin>::set_low(self);
    }

    pub fn set_high(&mut self) {
        // All architectures are infallible.
        let _ = <Self as OutputPin>::set_high(self);
    }

    pub fn toggle(&mut self) {
        // All architectures are infallible.
        let _ = <Self as StatefulOutputPin>::toggle(self);
    }
}

pub struct OutputBuilder<P: Peripheral<P: ArchOutputPin>> {
    pin: P,
    initial_state: PinState,
    drive_strength: DriveStrength,
    speed: Speed,
}

// TODO: should this be marked non_exaustive?
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum DriveStrength {
    Arch(ArchDriveStrength),
    Lowest,
    // Reset value of most GPIOs.
    Standard,
    Medium,
    High,
    Highest,
}

impl Default for DriveStrength {
    fn default() -> Self {
        Self::Standard
    }
}

// We introduce our own trait instead of using `From` because this conversion is not
// value-preserving.
pub trait FromDriveStrength {
    fn from(drive_strength: DriveStrength) -> ArchDriveStrength;
}

#[doc(alias = "SlewRate")]
#[derive(Copy, Clone, PartialEq, Eq)]
// FIXME: should we call this slew rate instead?
pub enum Speed {
    Arch(ArchSpeed),
    Low,
    Medium,
    High,
    VeryHigh,
}

impl Default for Speed {
    fn default() -> Self {
        Self::Low
    }
}

// We introduce our own trait instead of using `From` because this conversion is not
// value-preserving.
pub trait FromSpeed {
    fn from(speed: Speed) -> ArchSpeed;
}

impl<P: Peripheral<P: ArchOutputPin> + 'static> OutputBuilder<P> {
    pub fn drive_strength(self, drive_strength: DriveStrength) -> Self {
        const {
            assert!(
                arch::gpio::output::DRIVE_STRENGTH_AVAILABLE,
                "This architecture does not support setting the drive strength of GPIO outputs."
            );
        }

        Self {
            drive_strength,
            ..self
        }
    }

    // It is unclear whether `opt_*()` functions are actually useful, so we provide them but do not
    // commit to them being part of our API for now.
    // We may remove them in the future if we realize they are never useful.
    #[doc(hidden)]
    // TODO: or `drive_strength_opt`?
    pub fn opt_drive_strength(self, drive_strength: DriveStrength) -> Self {
        if arch::gpio::output::DRIVE_STRENGTH_AVAILABLE {
            // We cannot reuse the non-`opt_*()`, otherwise the const assert inside it would always
            // be triggered.
            Self {
                drive_strength,
                ..self
            }
        } else {
            self
        }
    }

    pub fn speed(self, speed: Speed) -> Self {
        const {
            assert!(
                arch::gpio::output::SPEED_AVAILABLE,
                "This architecture does not support setting the speed of GPIO outputs."
            );
        }

        Self { speed, ..self }
    }

    // It is unclear whether `opt_*()` functions are actually useful, so we provide them but do not
    // commit to them being part of our API for now.
    // We may remove them in the future if we realize they are never useful.
    #[doc(hidden)]
    // TODO: or `speed_opt`?
    pub fn opt_speed(self, speed: Speed) -> Self {
        if arch::gpio::output::SPEED_AVAILABLE {
            // We cannot reuse the non-`opt_*()`, otherwise the const assert inside it would always
            // be triggered.
            Self { speed, ..self }
        } else {
            self
        }
    }

    pub fn build(self) -> Output {
        // TODO: should we move this into `output::new()`s?
        let drive_strength = <ArchDriveStrength as FromDriveStrength>::from(self.drive_strength);
        // TODO: should we move this into `output::new()`s?
        let speed = <ArchSpeed as FromSpeed>::from(self.speed);

        let output = arch::gpio::output::new(self.pin, self.initial_state, drive_strength, speed);

        Output { output }
    }
}

impl embedded_hal::digital::ErrorType for Output {
    type Error = <ArchOutput<'static> as embedded_hal::digital::ErrorType>::Error;
}

impl OutputPin for Output {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        <ArchOutput as OutputPin>::set_low(&mut self.output)
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        <ArchOutput as OutputPin>::set_high(&mut self.output)
    }
}

// Outputs are all stateful outputs on:
// - embassy-nrf
// - embassy-rp
// - esp-hal
// - embassy-stm32
impl StatefulOutputPin for Output {
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        <ArchOutput as StatefulOutputPin>::is_set_high(&mut self.output)
    }

    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        <ArchOutput as StatefulOutputPin>::is_set_low(&mut self.output)
    }
}
