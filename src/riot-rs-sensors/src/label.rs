/// Label of a [`Value`](crate::sensor::Value) part of a
/// [`Values`](crate::sensor::Values) tuple.
///
/// # For sensor driver implementors
///
/// [`Label::Main`] must be used for sensor drivers returning a single
/// [`Value`](crate::sensor::Value), even if a more specific label exists for the
/// physical quantity.
/// This allows consumers displaying the label to ignore it for sensor drivers returning a single
/// [`Value`](crate::sensor::Value).
/// Other labels are reserved for sensor drivers returning multiple physical quantities.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Label {
    /// Used for sensor drivers returning a single [`Value`](crate::sensor::Value).
    Main,
    /// Humidity.
    Humidity,
    /// Temperature.
    Temperature,
    /// X axis.
    X,
    /// Y axis.
    Y,
    /// Z axis.
    Z,
}

impl core::fmt::Display for Label {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Main => write!(f, ""),
            Self::Humidity => write!(f, "Humidity"),
            Self::Temperature => write!(f, "Temperature"),
            Self::X => write!(f, "X"),
            Self::Y => write!(f, "Y"),
            Self::Z => write!(f, "Z"),
        }
    }
}
