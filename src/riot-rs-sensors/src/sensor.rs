//! Provides a [`Sensor`] trait abstracting over implementation details of a sensor driver.
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::ReceiveFuture};

use crate::{Category, Label, MeasurementUnit};

pub use crate::{
    value::{Accuracy, Value},
    Reading,
};

riot_rs_macros::define_count_adjusted_sensor_enums!();

/// This trait must be implemented by sensor drivers.
///
/// See [the module level documentation](crate) for more.
pub trait Sensor: Send + Sync {
    /// Triggers a measurement.
    /// Clears the previous reading.
    ///
    /// To obtain readings from every sensor drivers this method can be called in a loop over all
    /// sensors returned by [`Registry::sensors()`](crate::registry::Registry::sensors), before
    /// obtaining the readings with [`Self::wait_for_reading()`], so that the measurements happen
    /// concurrently.
    ///
    /// # For implementors
    ///
    /// This method should return quickly.
    ///
    /// # Errors
    ///
    /// Returns [`TriggerMeasurementError::NonEnabled`] if the sensor driver is not enabled.
    fn trigger_measurement(&self) -> Result<(), TriggerMeasurementError>;

    /// Waits for the reading and returns it asynchronously.
    /// Depending on the sensor device and the sensor driver, this may use a sensor interrupt or
    /// data polling.
    ///
    /// Interpretation of the reading requires data from [`Sensor::reading_axes()`] as well.
    /// See [the module level documentation](crate) for more.
    ///
    /// # Errors
    ///
    /// - Quickly returns [`ReadingError::NonEnabled`] if the sensor driver is not enabled.
    /// - Returns [`ReadingError::SensorAccess`] if the sensor device cannot be accessed.
    fn wait_for_reading(&'static self) -> ReadingWaiter;

    /// Provides information about the reading returned by [`Sensor::wait_for_reading()`].
    #[must_use]
    fn reading_axes(&self) -> ReadingAxes;

    /// Sets the sensor driver mode and returns the previous state.
    /// Allows to put the sensor device to sleep if supported.
    ///
    /// # Errors
    ///
    /// Returns [`SetModeError::Uninitialized`] if the sensor driver is not initialized.
    fn set_mode(&self, mode: Mode) -> Result<State, SetModeError>;

    /// Returns the current sensor driver state.
    #[must_use]
    fn state(&self) -> State;

    /// Returns the categories the sensor device is part of.
    #[must_use]
    fn categories(&self) -> &'static [Category];

    /// String label of the sensor driver *instance*.
    /// For instance, in the case of a temperature sensor, this allows to specify whether this
    /// specific sensor device is placed indoor or outdoor.
    #[must_use]
    fn label(&self) -> Option<&'static str>;

    /// Returns a human-readable name of the *sensor driver*.
    /// For instance, "push button" and "3-axis accelerometer" are appropriate display names.
    ///
    /// # Note
    ///
    /// Different sensor drivers for the same sensor device may have different display names.
    #[must_use]
    fn display_name(&self) -> Option<&'static str>;

    /// Returns the sensor device part number.
    /// Returns `None` when the sensor device does not have a part number.
    /// For instance, "DS18B20" is a valid part number.
    #[must_use]
    fn part_number(&self) -> Option<&'static str>;

    /// Returns the sensor driver version number.
    #[must_use]
    fn version(&self) -> u8;
}

/// Future returned by [`Sensor::wait_for_reading()`].
#[must_use = "futures do nothing unless you `.await` or poll them"]
#[pin_project::pin_project(project = ReadingWaiterProj)]
pub enum ReadingWaiter {
    #[doc(hidden)]
    Waiter {
        #[pin]
        waiter: ReceiveFuture<'static, CriticalSectionRawMutex, ReadingResult<Values>, 1>,
    },
    #[doc(hidden)]
    Err(ReadingError),
    #[doc(hidden)]
    Resolved,
}

impl Future for ReadingWaiter {
    type Output = ReadingResult<Values>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().project();
        match this {
            ReadingWaiterProj::Waiter { waiter } => waiter.poll(cx),
            ReadingWaiterProj::Err(err) => {
                // Replace the error with a dummy error value, crafted from thin air, and mark the
                // future as resolved, so that we do not take this dummy value into account later.
                // This avoids requiring `Clone` on `ReadingError`.
                let err = core::mem::replace(err, ReadingError::NonEnabled);
                *self = ReadingWaiter::Resolved;

                Poll::Ready(Err(err))
            }
            ReadingWaiterProj::Resolved => unreachable!(),
        }
    }
}

/// Mode of a sensor driver.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Mode {
    /// The sensor driver is disabled.
    Disabled,
    /// The sensor driver is enabled.
    Enabled,
    /// The sensor driver is sleeping.
    /// The sensor device may be in a low-power mode.
    Sleeping,
}

/// Possible errors when attempting to set the mode of a sensor driver.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SetModeError {
    /// The sensor driver is uninitialized.
    /// It has not been initialized yet, or initialization could not succeed.
    Uninitialized,
}

impl core::fmt::Display for SetModeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Uninitialized => write!(f, "sensor driver is not initialized"),
        }
    }
}

impl core::error::Error for SetModeError {}

/// State of a sensor driver.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum State {
    /// The sensor driver is uninitialized.
    /// It has not been initialized yet, or initialization could not succeed.
    #[default]
    Uninitialized = 0,
    /// The sensor driver is disabled.
    Disabled = 1,
    /// The sensor driver is enabled.
    Enabled = 2,
    /// The sensor driver is sleeping.
    Sleeping = 3,
}

impl From<Mode> for State {
    fn from(mode: Mode) -> Self {
        match mode {
            Mode::Disabled => Self::Disabled,
            Mode::Enabled => Self::Enabled,
            Mode::Sleeping => Self::Sleeping,
        }
    }
}

impl TryFrom<u8> for State {
    type Error = TryFromIntError;

    fn try_from(int: u8) -> Result<Self, Self::Error> {
        match int {
            0 => Ok(State::Uninitialized),
            1 => Ok(State::Disabled),
            2 => Ok(State::Enabled),
            3 => Ok(State::Sleeping),
            _ => Err(TryFromIntError),
        }
    }
}

/// The error type returned when a checked integral type conversion fails.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct TryFromIntError;

impl core::fmt::Display for TryFromIntError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "out of range integral type conversion attempted")
    }
}

impl core::error::Error for TryFromIntError {}

/// Provides metadata about a [`Value`].
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// NOTE(derive): we do not implement `Eq` on purpose: its would prevent us from possibly adding
// floats in the future.
pub struct ReadingAxis {
    label: Label,
    scaling: i8,
    unit: MeasurementUnit,
    accuracy: AccuracyFn,
}

impl ReadingAxis {
    /// Creates a new [`ReadingAxis`].
    ///
    /// This constructor is intended for sensor driver implementors only.
    #[must_use]
    pub fn new(label: Label, scaling: i8, unit: MeasurementUnit, accuracy: AccuracyFn) -> Self {
        Self {
            label,
            scaling,
            unit,
            accuracy,
        }
    }

    /// Returns the [`Label`] for this axis.
    #[must_use]
    pub fn label(&self) -> Label {
        self.label
    }

    /// Returns the [scaling](Value) for this axis.
    #[must_use]
    pub fn scaling(&self) -> i8 {
        self.scaling
    }

    /// Returns the unit of measurement for this axis.
    #[must_use]
    pub fn unit(&self) -> MeasurementUnit {
        self.unit
    }

    /// Returns a function allowing to obtain the accuracy error of a recently obtained
    /// [`Value`].
    ///
    /// # Note
    ///
    /// As the accuracy may depend on the sensor driver configuration, that accuracy function
    /// should only be used for one [`Value`] instance, and it is necessary to obtain an
    /// up-to-date function through an up-to-date [`ReadingAxis`].
    #[must_use]
    pub fn accuracy_fn(&self) -> AccuracyFn {
        self.accuracy
    }
}

/// Function allowing to obtain the accuracy error of a [`Value`], returned by
/// [`ReadingAxis::accuracy_fn()`].
pub type AccuracyFn = fn(Value) -> Accuracy;

/// Represents errors happening when *triggering* a sensor measurement.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TriggerMeasurementError {
    /// The sensor driver is not enabled (e.g., it may be disabled or sleeping).
    NonEnabled,
}

impl core::fmt::Display for TriggerMeasurementError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NonEnabled => write!(f, "sensor driver is not enabled"),
        }
    }
}

impl core::error::Error for TriggerMeasurementError {}

/// Represents errors happening when accessing a sensor reading.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ReadingError {
    /// The sensor driver is not enabled (e.g., it may be disabled or sleeping).
    NonEnabled,
    /// Cannot access the sensor device (e.g., because of a bus error).
    SensorAccess,
}

impl core::fmt::Display for ReadingError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NonEnabled => write!(f, "sensor driver is not enabled"),
            Self::SensorAccess => write!(f, "sensor device could not be accessed"),
        }
    }
}

impl core::error::Error for ReadingError {}

/// A specialized [`Result`] type for [`Reading`] operations.
pub type ReadingResult<R> = Result<R, ReadingError>;

#[cfg(test)]
mod tests {
    use super::*;

    // Assert that the Sensor trait is object-safe
    static _SENSOR_REFS: &[&dyn Sensor] = &[];
}
