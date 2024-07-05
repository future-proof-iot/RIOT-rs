use embedded_hal::digital::{OutputPin, StatefulOutputPin};

use crate::arch::{
    self,
    gpio::{
        input::{Input as ArchInput, Pin as ArchInputPin},
        output::{
            DriveStrength as ArchDriveStrength, OpenDrainOutput as ArchOpenDrainOutput,
            Output as ArchOutput, Pin as ArchOutputPin, Speed as ArchSpeed,
        },
    },
    peripheral::Peripheral,
};

pub use embedded_hal::digital::PinState;

pub struct Input {
    input: ArchInput<'static>, // FIXME: is this ok to require a 'static pin?
}

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

impl embedded_hal::digital::ErrorType for Input {
    type Error = <ArchInput<'static> as embedded_hal::digital::ErrorType>::Error;
}

pub struct IntEnabledInput {
    input: ArchInput<'static>, // FIXME: is this ok to require a 'static pin?
}

impl IntEnabledInput {
    pub async fn wait_for_high(&mut self) {
        self.input.wait_for_high().await;
    }

    pub async fn wait_for_low(&mut self) {
        self.input.wait_for_low().await;
    }

    pub async fn wait_for_rising_edge(&mut self) {
        self.input.wait_for_rising_edge().await;
    }

    pub async fn wait_for_falling_edge(&mut self) {
        self.input.wait_for_falling_edge().await;
    }

    pub async fn wait_for_any_edge(&mut self) {
        self.input.wait_for_any_edge().await;
    }
}

impl embedded_hal::digital::ErrorType for IntEnabledInput {
    type Error = <ArchInput<'static> as embedded_hal::digital::ErrorType>::Error;
}

impl embedded_hal_async::digital::Wait for IntEnabledInput {
    async fn wait_for_high(&mut self) -> Result<(), Self::Error> {
        <ArchInput as embedded_hal_async::digital::Wait>::wait_for_high(&mut self.input).await
    }

    async fn wait_for_low(&mut self) -> Result<(), Self::Error> {
        <ArchInput as embedded_hal_async::digital::Wait>::wait_for_low(&mut self.input).await
    }

    async fn wait_for_rising_edge(&mut self) -> Result<(), Self::Error> {
        <ArchInput as embedded_hal_async::digital::Wait>::wait_for_rising_edge(&mut self.input)
            .await
    }

    async fn wait_for_falling_edge(&mut self) -> Result<(), Self::Error> {
        <ArchInput as embedded_hal_async::digital::Wait>::wait_for_falling_edge(&mut self.input)
            .await
    }

    async fn wait_for_any_edge(&mut self) -> Result<(), Self::Error> {
        <ArchInput as embedded_hal_async::digital::Wait>::wait_for_any_edge(&mut self.input).await
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
        let input = match arch::gpio::input::new(self.pin, false, self.pull, self.schmitt_trigger) {
            Ok(input) => input,
            Err(input::Error::InterruptChannel(_)) => unreachable!(),
        };

        Input { input }
    }

    // FIXME: rename this
    pub fn build_with_interrupt(self) -> Result<IntEnabledInput, input::Error> {
        let input = arch::gpio::input::new(self.pin, true, self.pull, self.schmitt_trigger)?;

        Ok(IntEnabledInput { input })
    }
}

pub mod input {
    use crate::extint_registry;

    // TODO: rename this or move this to a sub-module
    #[derive(Debug)]
    pub enum Error {
        InterruptChannel(extint_registry::Error),
    }

    impl From<extint_registry::Error> for Error {
        fn from(err: extint_registry::Error) -> Self {
            Error::InterruptChannel(err)
        }
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

// FIXME: rename it to OutputOpenDrain for consistency with Embassy?
pub struct OpenDrainOutput {
    output: ArchOpenDrainOutput<'static>, // FIXME: is this ok to require a 'static pin?
}

impl OpenDrainOutput {
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
pub(crate) trait FromDriveStrength {
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
pub(crate) trait FromSpeed {
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

    pub fn build_open_drain(self) -> OpenDrainOutput {
        // It is not clear whether any architectures does *not* support open-drain, but we still
        // check it for forward compatibility.
        const {
            assert!(
                arch::gpio::output::OPEN_DRAIN_AVAILABLE,
                "This architecture does not support open-drain GPIO outputs."
            );
        }

        // TODO: should we move this into `output::new()`s?
        let drive_strength = <ArchDriveStrength as FromDriveStrength>::from(self.drive_strength);
        // TODO: should we move this into `output::new()`s?
        let speed = <ArchSpeed as FromSpeed>::from(self.speed);

        let output =
            arch::gpio::output::new_open_drain(self.pin, self.initial_state, drive_strength, speed);

        OpenDrainOutput { output }
    }
}

macro_rules! impl_embedded_hal_output_traits {
    ($type:ident, $arch_type:ident) => {
        impl embedded_hal::digital::ErrorType for $type {
            type Error = <$arch_type<'static> as embedded_hal::digital::ErrorType>::Error;
        }

        impl OutputPin for $type {
            fn set_low(&mut self) -> Result<(), Self::Error> {
                <$arch_type as OutputPin>::set_low(&mut self.output)
            }

            fn set_high(&mut self) -> Result<(), Self::Error> {
                <$arch_type as OutputPin>::set_high(&mut self.output)
            }
        }

        // Outputs are all stateful outputs on:
        // - embassy-nrf
        // - embassy-rp
        // - esp-hal
        // - embassy-stm32
        impl StatefulOutputPin for $type {
            fn is_set_high(&mut self) -> Result<bool, Self::Error> {
                <$arch_type as StatefulOutputPin>::is_set_high(&mut self.output)
            }

            fn is_set_low(&mut self) -> Result<bool, Self::Error> {
                <$arch_type as StatefulOutputPin>::is_set_low(&mut self.output)
            }
        }
    };
}

impl_embedded_hal_output_traits!(Output, ArchOutput);
impl_embedded_hal_output_traits!(OpenDrainOutput, ArchOpenDrainOutput);
