/// Represents a unit of measurement.
// Built upon https://doc.riot-os.org/phydat_8h_source.html
// and https://bthome.io/format/#sensor-data
// and https://www.rfc-editor.org/rfc/rfc8798.html
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum MeasurementUnit {
    /// [Acceleration *g*](https://en.wikipedia.org/wiki/G-force#Unit_and_measurement).
    AccelG,
    /// Value one represents an active state (e.g., a push button being pressed).
    ActiveOne,
    /// Value zero represents an active state (e.g., a push button being pressed).
    ActiveZero,
    /// Logic boolean.
    Bool,
    /// Degree Celsius.
    Celsius,
    /// Percent.
    Percent,
}

impl core::fmt::Display for MeasurementUnit {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::AccelG => write!(f, "g"),
            Self::ActiveOne | Self::ActiveZero | Self::Bool => write!(f, ""),
            Self::Celsius => write!(f, "°C"), // The Unicode Standard v15 recommends using U+00B0 + U+0043.
            Self::Percent => write!(f, "%"),  // TODO: should we have a different unit for %RH?
        }
    }
}
