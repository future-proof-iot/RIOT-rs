use embedded_hal::digital::{OutputPin, StatefulOutputPin};

use crate::arch::{
    self,
    gpio::{
        output::{DriveStrength as ArchDriveStrength, Output as ArchOutput},
        Pin as ArchPin,
    },
};

pub use embedded_hal::digital::PinState;

pub struct Output {
    output: ArchOutput<'static>, // FIXME: is this ok to require a 'static pin?
}

impl Output {
    pub fn new(pin: impl ArchPin, initial_state: PinState) -> Self {
        Self::builder(pin, initial_state).build()
    }

    pub fn builder<P: ArchPin>(pin: P, initial_state: PinState) -> OutputBuilder<P> {
        OutputBuilder {
            pin,
            initial_state,
            drive_strength: DriveStrength::Standard,
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

pub struct OutputBuilder<P: ArchPin> {
    pin: P,
    initial_state: PinState,
    drive_strength: DriveStrength,
}

// TODO: should this be marked non_exaustive?
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum DriveStrength {
    Arch(ArchDriveStrength),
    Lowest,
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

impl<P: ArchPin> OutputBuilder<P> {
    pub fn drive_strength(self, drive_strength: DriveStrength) -> Self {
        Self {
            drive_strength,
            ..self
        }
    }

    pub fn build(self) -> Output {
        // TODO: should me move this into `output::new()`s?
        let drive_strength = <ArchDriveStrength as FromDriveStrength>::from(self.drive_strength);

        let output = arch::gpio::output::new(self.pin, self.initial_state, drive_strength);

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
