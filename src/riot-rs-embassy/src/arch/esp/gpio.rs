use crate::arch;

pub fn init(peripherals: &mut arch::OptionalPeripherals) {
    let io = esp_hal::gpio::Io::new(
        peripherals.GPIO.take().unwrap(),
        peripherals.IO_MUX.take().unwrap(),
    );
    let pins = io.pins;

    peripherals.Gpio0.replace(pins.gpio0);
    peripherals.Gpio1.replace(pins.gpio1);
    peripherals.Gpio2.replace(pins.gpio2);
    peripherals.Gpio3.replace(pins.gpio3);
    peripherals.Gpio4.replace(pins.gpio4);
    peripherals.Gpio5.replace(pins.gpio5);
    peripherals.Gpio6.replace(pins.gpio6);
    peripherals.Gpio7.replace(pins.gpio7);
    peripherals.Gpio8.replace(pins.gpio8);
    peripherals.Gpio9.replace(pins.gpio9);
    peripherals.Gpio10.replace(pins.gpio10);
    peripherals.Gpio11.replace(pins.gpio11);
    peripherals.Gpio12.replace(pins.gpio12);
    peripherals.Gpio13.replace(pins.gpio13);
    peripherals.Gpio14.replace(pins.gpio14);
    peripherals.Gpio15.replace(pins.gpio15);
    peripherals.Gpio16.replace(pins.gpio16);
    peripherals.Gpio17.replace(pins.gpio17);
    peripherals.Gpio18.replace(pins.gpio18);
    peripherals.Gpio19.replace(pins.gpio19);
    peripherals.Gpio20.replace(pins.gpio20);
    peripherals.Gpio21.replace(pins.gpio21);
    peripherals.Gpio22.replace(pins.gpio22);
    peripherals.Gpio23.replace(pins.gpio23);
    peripherals.Gpio24.replace(pins.gpio24);
    peripherals.Gpio25.replace(pins.gpio25);
    peripherals.Gpio26.replace(pins.gpio26);
    peripherals.Gpio27.replace(pins.gpio27);
    peripherals.Gpio28.replace(pins.gpio28);
    peripherals.Gpio29.replace(pins.gpio29);
    peripherals.Gpio30.replace(pins.gpio30);
}

pub mod input {
    use esp_hal::gpio::{CreateErasedPin, InputPin, Level, Pull};

    use crate::{arch::peripheral::Peripheral, gpio};

    pub(crate) use esp_hal::gpio::AnyInput as Input;

    pub(crate) const SCHMITT_TRIGGER_AVAILABLE: bool = false;

    // NOTE(unstable-feature(trait_alias)): we may not have to use that unstable feature if we
    // define our own Pin trait and implement it on all GPIO types.
    // TODO: ask upstream whether it's acceptable to use `CreateErasedPin` in this scenario
    pub trait InputPin = EspInputPin + CreateErasedPin;

    pub(crate) fn new(
        pin: impl Peripheral<P: Pin> + 'static,
        int_enabled: bool,
        pull: crate::gpio::Pull,
        _schmitt_trigger: bool, // Not supported by this architecture
    ) -> Result<Input<'static>, gpio::input::Error> {
        let pull = Pull::from(pull);

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
    use esp_hal::gpio::{CreateErasedPin, Level, OutputPin, Pull};

    use crate::{
        arch::peripheral::Peripheral,
        gpio::{FromDriveStrength, FromSpeed, PinState},
    };

    pub(crate) use esp_hal::gpio::AnyOutput as Output;

    // FIXME: ESP32 *does* support setting the drive strength, but esp-hal seems to currently make
    // this impossible on `AnyOutput` (unlike on `Output`), because it internally uses an
    // `ErasedPin`.
    pub(crate) const DRIVE_STRENGTH_AVAILABLE: bool = false;
    pub(crate) const SPEED_AVAILABLE: bool = false;

    pub trait OutputPin = EspOutputPin + CreateErasedPin;

    pub(crate) fn new(
        pin: impl Peripheral<P: Pin> + 'static,
        initial_state: PinState,
        drive_strength: DriveStrength,
        _speed: Speed, // Not supported by this architecture
    ) -> Output<'static> {
        let initial_state: bool = initial_state.into();
        let initial_state = Level::from(initial_state);
        let mut output = Output::new(pin, initial_state);
        // TODO
        // output.set_drive_strength(drive_strength.into());
        output
    }

    // We do not provide a `Default` impl as not all pins have the same reset value.
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub enum DriveStrength {
        _5mA,
        _10mA,
        _20mA,
        _40mA,
    }

    impl FromDriveStrength for DriveStrength {
        fn from(drive_strength: crate::gpio::DriveStrength) -> Self {
            use crate::gpio::DriveStrength::*;

            // ESPs are able to output up to 40 mA, so we somewhat normalize this.
            match drive_strength {
                Arch(drive_strength) => drive_strength,
                Lowest => DriveStrength::_5mA,
                Standard => DriveStrength::_10mA,
                Medium => DriveStrength::_10mA,
                High => DriveStrength::_20mA,
                Highest => DriveStrength::_40mA,
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

// FIXME: keep name consistent with OpenDrainOutput
pub mod open_drain_output {
    use esp_hal::gpio::{CreateErasedPin, InputPin, OutputPin, Level, Pull};

    use crate::{
        arch::peripheral::Peripheral,
        gpio::{FromDriveStrength, FromSpeed, PinState},
    };

    use super::output::{DriveStrength, Speed};

    pub(crate) use esp_hal::gpio::AnyOutputOpenDrain as OpenDrainOutput;

    pub(crate) trait Pin = OutputPin + InputPin + CreateErasedPin;

    pub(crate) fn new(
        pin: impl Peripheral<P: Pin> + 'static,
        initial_state: PinState,
        drive_strength: DriveStrength,
        pull: crate::gpio::Pull,
        _speed: Speed, // Not supported by this architecture
    ) -> OpenDrainOutput<'static> {
        let initial_state: bool = initial_state.into();
        let initial_state = Level::from(initial_state);
        let pull = Pull::from(pull);
        let mut output = OpenDrainOutput::new(pin, initial_state, pull);
        // TODO
        // output.set_drive_strength(drive_strength.into());
        output
    }
}
