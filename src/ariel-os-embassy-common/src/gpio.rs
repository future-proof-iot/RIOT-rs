//! Provides HAL-agnostic GPIO-related types.

/// Digital level of an input or output.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Level {
    /// Digital low level.
    Low,
    /// Digital high level.
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
        if boolean {
            Level::High
        } else {
            Level::Low
        }
    }
}

impl From<embedded_hal::digital::PinState> for Level {
    fn from(pin_state: embedded_hal::digital::PinState) -> Self {
        bool::from(pin_state).into()
    }
}

impl From<Level> for embedded_hal::digital::PinState {
    fn from(level: Level) -> Self {
        bool::from(level).into()
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! define_into_level {
    () => {
        // The `Level` taken as parameter is the HAL-specific type.
        #[doc(hidden)]
        pub fn into_level(level: Level) -> $crate::gpio::Level {
            match level {
                Level::Low => $crate::gpio::Level::Low,
                Level::High => $crate::gpio::Level::High,
            }
        }
    };
}

/// Pull-up/pull-down resistor configuration.
///
/// All the MCU families we support have pull-up and pull-down resistors.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Pull {
    /// No pull-up or pull-down resistor.
    None,
    /// Pull-up resistor.
    Up,
    /// Pull-down resistor.
    Down,
}

#[doc(hidden)]
#[macro_export]
macro_rules! define_from_pull {
    () => {
        // The returned `Pull` is the HAL-specific type.
        fn from_pull(pull: $crate::gpio::Pull) -> Pull {
            match pull {
                $crate::gpio::Pull::None => Pull::None,
                $crate::gpio::Pull::Up => Pull::Up,
                $crate::gpio::Pull::Down => Pull::Down,
            }
        }
    };
}

/// Drive strength of an output.
///
/// This enum allows to either use high-level, portable values, roughly normalized across
/// HALs, or to use HAL-specific values if needed.
// TODO: should this be marked non_exhaustive?
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum DriveStrength<A> {
    /// HAL-specific drive strength setting.
    Hal(A),
    /// Lowest drive strength available on this HAL.
    Lowest,
    /// Most common reset value of drive strength on this HAL.
    Standard,
    /// Medium drive strength.
    Medium,
    /// High drive strength.
    High,
    /// Highest drive strength available on this HAL.
    Highest,
}

impl<A> Default for DriveStrength<A> {
    fn default() -> Self {
        Self::Standard
    }
}

// We introduce our own trait instead of using `From` because this conversion is not
// value-preserving.
#[doc(hidden)]
pub trait FromDriveStrength {
    /// Converts the HAL-agnostic type to a HAL-specific type.
    fn from(drive_strength: DriveStrength<Self>) -> Self
    where
        Self: Sized;
}

/// Speed setting of an output.
///
/// Speed can be increased when needed, at the price of increasing high-frequency noise.
///
/// This enum allows to either use high-level, portable values, roughly normalized across
/// HALs, or to use HAL-specific values if needed.
#[doc(alias = "SlewRate")]
#[derive(Copy, Clone, PartialEq, Eq)]
// FIXME: should we call this slew rate instead?
pub enum Speed<A> {
    /// HAL-specific speed setting.
    Hal(A),
    /// Low speed.
    Low,
    /// Medium speed.
    Medium,
    /// High speed.
    High,
    /// Very high speed.
    VeryHigh,
}

impl<A> Default for Speed<A> {
    fn default() -> Self {
        Self::Low
    }
}

// We introduce our own trait instead of using `From` because this conversion is not
// value-preserving.
#[doc(hidden)]
pub trait FromSpeed {
    /// Converts the HAL-agnostic type to a HAL-specific type.
    fn from(speed: Speed<Self>) -> Self
    where
        Self: Sized;
}

pub mod input {
    //! Input-specific types.

    /// Input-related errors.
    #[derive(Debug)]
    pub enum Error {
        /// Error when hitting hardware limitations regarding interrupt registration.
        #[cfg(feature = "external-interrupts")]
        InterruptChannel(InterruptError),
    }

    #[cfg(feature = "external-interrupts")]
    impl From<InterruptError> for Error {
        fn from(err: InterruptError) -> Self {
            Error::InterruptChannel(err)
        }
    }

    // FIXME(doc): document the variants
    /// External interrupt-related errors.
    ///
    /// Not all variants can happen on every HALs.
    #[cfg(feature = "external-interrupts")]
    #[derive(Debug)]
    pub enum InterruptError {
        /// On MCU families where interrupt channels are shared between multiple input GPIOs (e.g,
        /// STM32), signals that the interrupt channel is already used by another input GPIO.
        IntChannelAlreadyUsed,
        /// On MCU families where there is a pool of interrupt channels, with fewer channels than
        /// input GPIOs, signals that no interrupt channel is left available.
        NoIntChannelAvailable,
    }
}

/// Available output speed/slew rate settings.
///
/// *Note: configuring the speed of outputs is not supported on this MCU family.*
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum UnsupportedSpeed {
    #[doc(hidden)]
    UnsupportedByHardware,
}

impl FromSpeed for UnsupportedSpeed {
    fn from(_speed: Speed<Self>) -> Self {
        Self::UnsupportedByHardware
    }
}

/// Available drive strength settings.
///
/// *Note: configuring the drive strength of outputs is not supported on this MCU family.*
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum UnsupportedDriveStrength {
    #[doc(hidden)]
    UnsupportedByHardware,
}

impl Default for UnsupportedDriveStrength {
    fn default() -> Self {
        Self::UnsupportedByHardware
    }
}

impl FromDriveStrength for UnsupportedDriveStrength {
    fn from(drive_strength: DriveStrength<Self>) -> Self {
        match drive_strength {
            DriveStrength::Hal(drive_strength) => drive_strength,
            DriveStrength::Lowest
            | DriveStrength::Medium
            | DriveStrength::High
            | DriveStrength::Highest => UnsupportedDriveStrength::UnsupportedByHardware,
            DriveStrength::Standard => UnsupportedDriveStrength::default(),
        }
    }
}
