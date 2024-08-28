pub trait IntoLevel {
    fn into(level: Self) -> riot_rs_embassy_common::gpio::Level;
}

pub fn init(peripherals: &mut crate::OptionalPeripherals) {
    let io = esp_hal::gpio::Io::new(
        peripherals.GPIO.take().unwrap(),
        peripherals.IO_MUX.take().unwrap(),
    );
    let pins = io.pins;

    peripherals.GPIO_0.replace(pins.gpio0);
    peripherals.GPIO_1.replace(pins.gpio1);
    peripherals.GPIO_2.replace(pins.gpio2);
    peripherals.GPIO_3.replace(pins.gpio3);
    peripherals.GPIO_4.replace(pins.gpio4);
    peripherals.GPIO_5.replace(pins.gpio5);
    peripherals.GPIO_6.replace(pins.gpio6);
    peripherals.GPIO_7.replace(pins.gpio7);
    peripherals.GPIO_8.replace(pins.gpio8);
    peripherals.GPIO_9.replace(pins.gpio9);
    peripherals.GPIO_10.replace(pins.gpio10);
    peripherals.GPIO_11.replace(pins.gpio11);
    peripherals.GPIO_12.replace(pins.gpio12);
    peripherals.GPIO_13.replace(pins.gpio13);
    peripherals.GPIO_14.replace(pins.gpio14);
    peripherals.GPIO_15.replace(pins.gpio15);
    peripherals.GPIO_16.replace(pins.gpio16);
    peripherals.GPIO_17.replace(pins.gpio17);
    peripherals.GPIO_18.replace(pins.gpio18);
    peripherals.GPIO_19.replace(pins.gpio19);
    peripherals.GPIO_20.replace(pins.gpio20);

    #[cfg(context = "esp32c6")]
    {
        peripherals.GPIO_21.replace(pins.gpio21);
        peripherals.GPIO_22.replace(pins.gpio22);
        peripherals.GPIO_23.replace(pins.gpio23);
        peripherals.GPIO_24.replace(pins.gpio24);
        peripherals.GPIO_25.replace(pins.gpio25);
        peripherals.GPIO_26.replace(pins.gpio26);
        peripherals.GPIO_27.replace(pins.gpio27);
        peripherals.GPIO_28.replace(pins.gpio28);
        peripherals.GPIO_29.replace(pins.gpio29);
        peripherals.GPIO_30.replace(pins.gpio30);
    }
}

pub mod input {
    use esp_hal::{
        gpio::{CreateErasedPin, InputPin as EspInputPin, Level, Pull},
        peripheral::Peripheral,
    };

    #[cfg(feature = "external-interrupts")]
    use riot_rs_embassy_common::gpio::input::InterruptError;

    pub use esp_hal::gpio::AnyInput as Input;

    // Re-export `AnyInput` as `IntEnabledInput` as they are interrupt-enabled.
    #[cfg(feature = "external-interrupts")]
    pub use esp_hal::gpio::AnyInput as IntEnabledInput;

    pub const SCHMITT_TRIGGER_CONFIGURABLE: bool = false;

    // NOTE(unstable-feature(trait_alias)): we may not have to use that unstable feature if we
    // define our own Pin trait and implement it on all GPIO types.
    // TODO: ask upstream whether it's acceptable to use `CreateErasedPin` in this scenario
    pub trait InputPin = EspInputPin + CreateErasedPin;

    pub fn new(
        pin: impl Peripheral<P: InputPin> + 'static,
        pull: riot_rs_embassy_common::gpio::Pull,
        _schmitt_trigger: bool, // Not supported by this architecture
    ) -> Result<Input<'static>, core::convert::Infallible> {
        let pull = from_pull(pull);

        Ok(Input::new(pin, pull))
    }

    #[cfg(feature = "external-interrupts")]
    pub fn new_int_enabled(
        pin: impl Peripheral<P: InputPin> + 'static,
        pull: riot_rs_embassy_common::gpio::Pull,
        _schmitt_trigger: bool, // Not supported by this architecture
    ) -> Result<IntEnabledInput<'static>, InterruptError> {
        match new(pin, pull, _schmitt_trigger) {
            Ok(input) => Ok(input),
            Err(err) => match err {
                // Compile-time check that this never happens as the Result is Infallible.
            },
        }
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
    use esp_hal::{
        gpio::{CreateErasedPin, Level, OutputPin as EspOutputPin},
        peripheral::Peripheral,
    };
    use riot_rs_embassy_common::gpio::{FromDriveStrength, FromSpeed};

    pub use esp_hal::gpio::AnyOutput as Output;

    // FIXME: ESP32 *does* support setting the drive strength, but esp-hal seems to currently make
    // this impossible on `AnyOutput` (unlike on `Output`), because it internally uses an
    // `ErasedPin`.
    pub const DRIVE_STRENGTH_CONFIGURABLE: bool = false;
    pub const SPEED_CONFIGURABLE: bool = false;

    pub trait OutputPin = EspOutputPin + CreateErasedPin;

    pub fn new(
        pin: impl Peripheral<P: OutputPin> + 'static,
        initial_level: riot_rs_embassy_common::gpio::Level,
        _drive_strength: DriveStrength,
        _speed: Speed, // Not supported by this architecture
    ) -> Output<'static> {
        let initial_level = match initial_level {
            riot_rs_embassy_common::gpio::Level::Low => Level::Low,
            riot_rs_embassy_common::gpio::Level::High => Level::High,
        };
        let output = Output::new(pin, initial_level);
        // TODO
        // output.set_drive_strength(drive_strength.into());
        output
    }

    // crate::gpio::impl_from_level!(Level);

    // We do not provide a `Default` impl as not all pins have the same reset value.
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub enum DriveStrength {
        _5mA,
        _10mA,
        _20mA,
        _40mA,
    }

    impl FromDriveStrength for DriveStrength {
        fn from(drive_strength: riot_rs_embassy_common::gpio::DriveStrength<Self>) -> Self {
            use riot_rs_embassy_common::gpio::DriveStrength::*;

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
        fn from(_speed: riot_rs_embassy_common::gpio::Speed<Self>) -> Self {
            Self::UnsupportedByArchitecture
        }
    }
}
