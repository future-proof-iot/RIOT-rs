//! Provides a sensor abstraction layer.
//!
//! # Definitions
//!
//! In the context of this abstraction:
//!
//! - A *sensor device* is a device measuring one or multiple physical quantities and reporting
//!   them as one or more digital values. - Sensor devices measuring the same physical quantity are
//!   said to be part of the same *sensor category*.
//!   A sensor device may be part of multiple sensor categories.
//! - A *measurement* is the physical operation of measuring one or several physical quantities.
//! - A *reading* is the digital result returned by a sensor device after carrying out a
//!   measurement.
//!   Values of different physical quantities can therefore be part of the same reading.
//! - A *sensor driver* refers to a sensor device as exposed by the sensor abstraction layer.
//! - A *sensor driver instance* is an instance of a sensor driver.
//!
//! # Accessing sensor driver instances
//!
//! Registered sensor driver instances can be accessed using
//! [`REGISTRY::sensors()`](registry::Registry::sensors).
//! Sensor drivers implement the [`Sensor`] trait, which allows to trigger measurements and obtain
//! the resulting readings.
//!
//! # Obtaining a sensor reading
//!
//! After triggering a measurement with [`Sensor::trigger_measurement()`], a reading can be
//! obtained using [`Sensor::wait_for_reading()`].
//! It is additionally necessary to use [`Sensor::reading_axes()`] to make sense of the obtained
//! reading:
//!
//! - [`Sensor::wait_for_reading()`] returns a [`Values`](sensor::Values), a data "tuple"
//!   containing values returned by the sensor driver.
//! - [`Sensor::reading_axes()`] returns a [`ReadingAxes`](sensor::ReadingAxes) which
//!   indicates which physical quantity each [`Value`](value::Value) from that tuple corresponds
//!   to, using a [`Label`].
//!   For instance, this allows to disambiguate the values provided by a temperature & humidity
//!   sensor.
//!   Each [`ReadingAxis`](sensor::ReadingAxis) also provides information about the
//!   measurement accuracy, through
//!   [`ReadingAxis::accuracy_fn()`](sensor::ReadingAxis::accuracy_fn).
//!
//! To avoid handling floats, [`Value`](value::Value)s returned by [`Sensor::wait_for_reading()`]
//! are integers, and a fixed scaling value is provided in [`ReadingAxis`](sensor::ReadingAxis),
//! for each [`Value`](value::Value) returned.
//! See [`Value`](value::Value) for more details.
//!
//! # For implementors
//!
//! Sensor drivers must implement the [`Sensor`] trait.
//!
#![no_std]
// Required by linkme
#![feature(used_with_arg)]
#![deny(clippy::pedantic)]
#![deny(missing_docs)]

mod category;
mod label;
mod measurement_unit;
pub mod registry;
pub mod sensor;
mod value;

pub use category::Category;
pub use label::Label;
pub use measurement_unit::MeasurementUnit;
pub use registry::{REGISTRY, SENSOR_REFS};
pub use sensor::Sensor;
pub use value::Reading;
