#[expect(clippy::doc_markdown)]
/// Represents a value obtained from a sensor device.
///
/// # Scaling
///
/// The [scaling value](crate::sensor::ReadingAxis::scaling()) obtained from the sensor driver with
/// [`Sensor::reading_axes()`](crate::Sensor::reading_axes) must be taken into account using the
/// following formula:
///
/// <math xmlns="http://www.w3.org/1998/Math/MathML" display="block"><mrow><mi mathvariant="monospace">Value::value()</mi></mrow><mo>·</mo><msup><mn>10</mn><mrow><mi mathvariant="monospace">scaling</mi></mrow></msup></math>
///
/// For instance, in the case of a temperature sensor, if [`Value::get()`] returns `2225` and the
/// scaling value is `-2`, this means that the temperature measured and returned by the sensor
/// device is `22.25` (the [measurement error](Accuracy) must additionally be taken into
/// account).
/// This is required to avoid handling floats.
///
/// # Unit of measurement
///
/// The unit of measurement can be obtained using
/// [`ReadingAxis::unit()`](crate::sensor::ReadingAxis::unit).
// NOTE(derive): we do not implement `Eq` or `PartialOrd` on purpose: `Eq` would prevent us from
// possibly adding floats in the future and `PartialOrd` does not make sense because interpreting
// the value requires the `ReadingAxis` associated with this `Value`.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Value {
    value: i32,
}

impl Value {
    /// Creates a new value.
    ///
    /// This constructor is intended for sensor driver implementors only.
    #[must_use]
    pub const fn new(value: i32) -> Self {
        Self { value }
    }

    /// Returns the value.
    #[must_use]
    pub fn get(&self) -> i32 {
        self.value
    }
}

/// Specifies the accuracy of a measurement.
///
/// The [`Accuracy`] should be obtained quickly after obtaining the [`Value`], as the
/// accuracy can be affected by a change in the internal state of the sensor driver.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Accuracy {
    /// Unknown accuracy.
    Unknown,
    /// No measurement error (e.g., boolean values from a push button).
    NoError,
    /// Measurement error symmetrical around the [`bias`](Accuracy::SymmetricalError::bias).
    ///
    /// The unit of measurement is provided by the [`ReadingAxis`](crate::sensor::ReadingAxis)
    /// associated to the [`Value`].
    /// The `scaling` value is used for both `deviation` and `bias`.
    /// The accuracy error is thus given by the following formulas:
    ///
    /// <math xmlns="http://www.w3.org/1998/Math/MathML" display="block"><mo>+</mo><mo>(</mo><mrow><mi mathvariant="monospace">bias</mi></mrow><mo>+</mo><mrow><mi mathvariant="monospace">deviation</mi></mrow><mo>)</mo><mo>·</mo><msup><mn>10</mn><mrow><mi mathvariant="monospace">scaling</mi></mrow></msup>/<mo>-</mo><mo>(</mo><mrow><mi mathvariant="monospace">bias</mi></mrow><mo>-</mo><mrow><mi mathvariant="monospace">deviation</mi></mrow><mo>)</mo><mo>·</mo><msup><mn>10</mn><mrow><mi mathvariant="monospace">scaling</mi></mrow></msup></math>
    ///
    /// # Examples
    ///
    /// The DS18B20 temperature sensor accuracy error is <mo>+</mo><mn>0.05</mn>/<mo>-</mo><mn>0.45</mn>
    /// at 20 °C (see Figure 1 of its datasheet).
    /// [`Accuracy`] would thus be the following:
    ///
    /// ```
    /// # use riot_rs_sensors::sensor::Accuracy;
    /// Accuracy::SymmetricalError {
    ///     deviation: 25,
    ///     bias: -20,
    ///     scaling: -2,
    /// }
    /// # ;
    /// ```
    SymmetricalError {
        /// Deviation around the bias value.
        deviation: i16,
        /// Bias (mean accuracy error).
        bias: i16,
        /// Scaling of [`deviation`](Accuracy::SymmetricalError::deviation) and
        /// [`bias`](Accuracy::SymmetricalError::bias).
        scaling: i8,
    },
}

/// Implemented on [`Values`](crate::sensor::Values), returned by
/// [`Sensor::wait_for_reading()`](crate::Sensor::wait_for_reading).
pub trait Reading: core::fmt::Debug {
    /// Returns the first value returned by [`Reading::values()`].
    fn value(&self) -> Value;

    /// Returns an iterator over [`Value`]s of a sensor reading.
    ///
    /// The order of [`Value`]s is not significant, but is fixed.
    ///
    /// # For implementors
    ///
    /// The default implementation must be overridden on types containing multiple
    /// [`Value`]s.
    fn values(&self) -> impl ExactSizeIterator<Item = Value> {
        [self.value()].into_iter()
    }
}
