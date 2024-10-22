/// Represents a unit of measurement.
///
/// # For sensor driver implementors
///
/// Missing variants can be added when required.
/// Please open an issue to discuss it.
// Built upon https://doc.riot-os.org/phydat_8h_source.html
// and https://bthome.io/format/#sensor-data
// and https://www.iana.org/assignments/senml/senml.xhtml
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum MeasurementUnit {
    /// [Acceleration *g*](https://en.wikipedia.org/wiki/G-force#Unit_and_measurement).
    AccelG,
    /// Value one represents an active state (e.g., a push button being pressed).
    ActiveOne,
    /// Value zero represents an active state (e.g., a push button being pressed).
    ActiveZero,
    /// Ampere (A).
    Ampere,
    /// Becquerel (Bq).
    Becquerel,
    /// Logic boolean.
    Bool,
    /// Candela (cd).
    Candela,
    /// Degrees Celsius (°C).
    Celsius,
    /// Coulomb (C).
    Coulomb,
    /// Decibel (dB).
    Decibel,
    /// Farad (F).
    Farad,
    // FIXME: Kilogram as well?
    /// Gram (g).
    Gram,
    /// Gray (Gy).
    Gray,
    /// Henry (H).
    Henry,
    /// Hertz (Hz).
    Hertz,
    /// Joule (J).
    Joule,
    /// Katal (kat).
    Katal,
    /// Kelvin (K).
    Kelvin,
    /// Lumen (lm).
    Lumen,
    /// Lux (lx).
    Lux,
    /// Meter (m)
    Meter,
    /// Mole (mol).
    Mole,
    /// Newton (N).
    Newton,
    /// Ohm (Ω).
    Ohm,
    /// Pascal (Pa).
    Pascal,
    /// Percent (%).
    Percent,
    /// %RH.
    PercentageRelativeHumidity,
    /// Radian (rad).
    Radian,
    /// Second (s).
    Second,
    /// Siemens (S).
    Siemens,
    /// Sievert (Sv).
    Sievert,
    /// Steradian (sr).
    Steradian,
    /// Tesla (T).
    Tesla,
    /// Volt (V).
    Volt,
    /// Watt (W).
    Watt,
    /// Weber (Wb).
    Weber,
}

impl core::fmt::Display for MeasurementUnit {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        #[expect(clippy::match_same_arms)]
        match self {
            Self::AccelG => write!(f, "g"),
            Self::ActiveOne => write!(f, ""),
            Self::ActiveZero => write!(f, ""),
            Self::Ampere => write!(f, "A"),
            Self::Becquerel => write!(f, "Bq"),
            Self::Bool => write!(f, ""),
            Self::Candela => write!(f, "cd"),
            Self::Celsius => write!(f, "°C"), // The Unicode Standard v15 recommends using U+00B0 + U+0043.
            Self::Coulomb => write!(f, "C"),
            Self::Decibel => write!(f, "dB"),
            Self::Farad => write!(f, "F"),
            Self::Gram => write!(f, "g"),
            Self::Gray => write!(f, "Gy"),
            Self::Henry => write!(f, "H"),
            Self::Hertz => write!(f, "Hz"),
            Self::Joule => write!(f, "J"),
            Self::Katal => write!(f, "kat"),
            Self::Kelvin => write!(f, "K"),
            Self::Lumen => write!(f, "lm"),
            Self::Lux => write!(f, "lx"),
            Self::Meter => write!(f, "m"),
            Self::Mole => write!(f, "mol"),
            Self::Newton => write!(f, "N"),
            Self::Ohm => write!(f, "Ω"),
            Self::Pascal => write!(f, "Pa"),
            Self::Percent => write!(f, "%"),
            Self::PercentageRelativeHumidity => write!(f, "%RH"),
            Self::Radian => write!(f, "rad"),
            Self::Second => write!(f, "s"),
            Self::Siemens => write!(f, "S"),
            Self::Sievert => write!(f, "Sv"),
            Self::Steradian => write!(f, "sr"),
            Self::Tesla => write!(f, "T"),
            Self::Volt => write!(f, "V"),
            Self::Watt => write!(f, "W"),
            Self::Weber => write!(f, "Wb"),
        }
    }
}
