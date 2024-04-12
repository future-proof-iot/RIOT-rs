use core::{any::Any, future::Future};

// TODO: use a zero-copy channel instead?
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Receiver};

/// Represents a device providing sensor readings.
// FIXME: add a test to make sure this trait is object-safe
pub trait Sensor: Send + Sync {
    // FIXME: allow to return multiple PhysicalValue (with Reading?, but keep it object-safe)
    /// Returns a sensor reading.
    ///
    /// Blocks until the reading is ready.
    /// In the future, we may provide an async version of this method, when it becomes possible to
    /// provide dispatchable async method in traits without an allocator.
    fn read(&self) -> ReadingResult<PhysicalValue>;

    fn set_enabled(&self, enabled: bool);

    #[must_use]
    fn enabled(&self) -> bool;

    fn set_threshold(&self, kind: ThresholdKind, value: PhysicalValue);

    // TODO: merge this with set_threshold?
    fn set_threshold_enabled(&self, kind: ThresholdKind, enabled: bool);

    // TODO: tune the channel size
    #[must_use]
    fn subscribe(&self) -> NotificationReceiver;

    // TODO: can we make this a trait const instead? may cause object safety issues
    #[must_use]
    fn value_scale() -> i8
    where
        Self: Sized;

    // TODO: can we make this a trait const instead? may cause object safety issues
    #[must_use]
    fn unit() -> PhysicalUnit
    where
        Self: Sized;

    // TODO: i18n?
    #[must_use]
    fn display_name() -> Option<&'static str>
    where
        Self: Sized;

    #[must_use]
    fn part_number() -> &'static str
    where
        Self: Sized;

    #[must_use]
    fn version() -> u8
    where
        Self: Sized;
}

pub trait Reading {
    fn value(&self) -> PhysicalValue;

    fn values(&self) -> impl ExactSizeIterator<Item = PhysicalValue> {
        [self.value()].into_iter()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ThresholdKind {
    Lower,
    Higher,
}

// TODO: should we pass the value as well? that may be difficult because of the required generics
#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Notification {
    ReadingAvailable,
    Threshold(ThresholdKind),
}

pub type NotificationReceiver<'a> = Receiver<'a, CriticalSectionRawMutex, Notification, 1>;

// TODO: is it more useful to indicate the error nature or whether it is temporary or permanent?
#[derive(Debug)]
pub enum ReadingError {
    // The sensor is disabled.
    Disabled,
}

impl core::fmt::Display for ReadingError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // FIXME: update this
        write!(f, "error when accessing a sensor reading")
    }
}

impl core::error::Error for ReadingError {}

pub type ReadingResult<R> = Result<R, ReadingError>;

// TODO: add a timestamp?
// TODO: provide new() + getters instead of making fields public?
#[derive(Debug, Copy, Clone)]
pub struct PhysicalValue {
    pub value: i32,
}

// Built upon https://doc.riot-os.org/phydat_8h_source.html
// and https://bthome.io/format/#sensor-data
// and https://www.rfc-editor.org/rfc/rfc8798.html
#[derive(Debug, Copy, Clone)]
#[non_exhaustive]
pub enum PhysicalUnit {
    Celsius,
    // TODO: add other units
}
