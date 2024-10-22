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
    /// obtaining the readings with [`Self::wait_for_reading()`] in a second loop, so that the
    /// measurements happen concurrently.
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
    /// Interpretation of the reading requires data from [`Sensor::reading_axes()`] as well.
    /// See [the module level documentation](crate) for more.
    ///
    /// # Note
    ///
    /// It is necessary to trigger a measurement by calling [`Sensor::trigger_measurement()`]
    /// beforehand, even if the sensor device carries out periodic measurements on its own.
    ///
    /// # Errors
    ///
    /// - Quickly returns [`ReadingError::NonEnabled`] if the sensor driver is not enabled.
    /// - Quickly returns [`ReadingError::NotMeasuring`] if no measurement has been triggered
    ///   beforehand using [`Sensor::trigger_measurement()`].
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
    /// The sensor driver is enabled and a measurement has been triggered.
    Measuring = 3,
    /// The sensor driver is sleeping.
    Sleeping = 4,
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
            0 => Ok(Self::Uninitialized),
            1 => Ok(Self::Disabled),
            2 => Ok(Self::Enabled),
            3 => Ok(Self::Measuring),
            4 => Ok(Self::Sleeping),
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
    accuracy: Accuracy,
}

impl ReadingAxis {
    /// Creates a new [`ReadingAxis`].
    ///
    /// This constructor is intended for sensor driver implementors only.
    #[must_use]
    pub fn new(label: Label, scaling: i8, unit: MeasurementUnit, accuracy: Accuracy) -> Self {
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

    /// Returns the accuracy of the most recent reading obtained with
    /// [`Sensor::wait_for_reading()`].
    /// Returns [`Accuracy::NoReading`] when no reading has been obtained yet.
    ///
    /// # Note
    ///
    /// As the accuracy depends on the reading and also on other internal conditions, the accuracy
    /// must be obtained anew for each reading.
    #[must_use]
    pub fn accuracy(&self) -> Accuracy {
        self.accuracy
    }
}

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
    /// No measurement has been triggered before waiting for a reading.
    /// It is necessary to call [`Sensor::trigger_measurement()`] before calling
    /// [`Sensor::wait_for_reading()`].
    NotMeasuring,
}

impl core::fmt::Display for ReadingError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NonEnabled => write!(f, "sensor driver is not enabled"),
            Self::SensorAccess => write!(f, "sensor device could not be accessed"),
            Self::NotMeasuring => write!(f, "no measurement has been triggered"),
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
