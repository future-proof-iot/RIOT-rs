/// Categories a sensor can be part of.
///
/// A sensor can be part of multiple categories.
// Built upon https://doc.riot-os.org/group__drivers__saul.html#ga8f2dfec7e99562dbe5d785467bb71bbb
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Category {
    /// Accelerometer.
    Accelerometer,
    /// Humidity sensor.
    Humidity,
    /// Humidity and temperature sensor.
    HumidityTemperature,
    /// Push button.
    PushButton,
    /// Temperature sensor.
    Temperature,
}
