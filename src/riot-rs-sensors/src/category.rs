/// Categories a sensor driver can be part of.
///
/// A sensor driver can be part of multiple categories.
///
/// # For sensor driver implementors
///
/// Missing variants can be added when required.
/// Please open an issue to discuss it.
// Built upon https://doc.riot-os.org/group__drivers__saul.html#ga8f2dfec7e99562dbe5d785467bb71bbb
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Category {
    /// Accelerometer.
    Accelerometer,
    /// Ammeter (ampere meter).
    Ammeter,
    /// CO₂ gas sensor.
    Co2Gas,
    /// Color sensor.
    Color,
    /// Gyroscope.
    Gyroscope,
    /// Humidity sensor.
    Humidity,
    /// Humidity and temperature sensor.
    HumidityTemperature,
    /// Light sensor.
    Light,
    /// Magnetometer.
    Magnetometer,
    /// pH sensor.
    Ph,
    /// Pressure sensor.
    Pressure,
    /// Push button.
    PushButton,
    /// Temperature sensor.
    Temperature,
    /// TVOC sensor.
    Tvoc,
    /// Voltage sensor.
    Voltage,
}
