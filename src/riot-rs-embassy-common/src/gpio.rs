//! Provides architecture-agnostic GPIO-related types.

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
        // The `Level` taken as parameter is the arch-specific type.
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
/// All the architectures we support have pull-up and pull-down resistors.
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
        // The returned `Pull` is the arch-specific type.
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
/// architectures, or to use architecture-specific values if needed.
// TODO: should this be marked non_exhaustive?
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum DriveStrength<A> {
    /// Architecture-specific drive strength setting.
    Arch(A),
    /// Lowest drive strength available on this architecture.
    Lowest,
    /// Most common reset value of drive strength on this architecture.
    Standard,
    /// Medium drive strength.
    Medium,
    /// High drive strength.
    High,
    /// Highest drive strength available on this architecture.
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
    /// Converts the arch-agnostic type to an arch-specific type.
    fn from(drive_strength: DriveStrength<Self>) -> Self
    where
        Self: Sized;
}

/// Speed setting of an output.
///
/// Speed can be increased when needed, at the price of increasing high-frequency noise.
///
/// This enum allows to either use high-level, portable values, roughly normalized across
/// architectures, or to use architecture-specific values if needed.
#[doc(alias = "SlewRate")]
#[derive(Copy, Clone, PartialEq, Eq)]
// FIXME: should we call this slew rate instead?
pub enum Speed<A> {
    /// Architecture-specific speed setting.
    Arch(A),
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
    /// Converts the arch-agnostic type to an arch-specific type.
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
    /// Not all variants can happen on every architecture.
    #[cfg(feature = "external-interrupts")]
    #[derive(Debug)]
    pub enum InterruptError {
        /// On architectures where interrupt channels are shared between multiple input GPIOs (e.g,
        /// STM32), signals that the interrupt channel is already used by another input GPIO.
        IntChannelAlreadyUsed,
        /// On architectures where there is a pool of interrupt channels, with fewer channels than
        /// input GPIOs, signals that no interrupt channel is left available.
        NoIntChannelAvailable,
    }
}
