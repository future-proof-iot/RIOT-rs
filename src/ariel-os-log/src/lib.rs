//! Provides logging facilities.

#![cfg_attr(not(test), no_std)]
#![feature(doc_auto_cfg)]
#![deny(missing_docs)]
#![deny(clippy::pedantic)]

#[cfg(feature = "defmt")]
pub mod defmt {
    //! Selected [`defmt`] items.

    // This module is hidden in the docs, but would still be imported by a wildcard import of this
    // crate's items.
    #[doc(hidden)]
    pub mod hidden {
        // Required so the macros can access it.
        #[doc(hidden)]
        pub use defmt;
    }

    pub use defmt::{unreachable, Debug2Format, Display2Format, Format};

    // These are required "internally" by `defmt`.
    pub use defmt::{export, Formatter, Str};
}

// The declarative macros are required because the defmt macros expect defmt to be in scope.

/// Logs a message at the trace level.
#[cfg(feature = "defmt")]
#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {{
        use $crate::defmt::hidden::defmt;
        defmt::trace!($($arg)*);
    }};
}

/// Logs a message at the debug level.
#[cfg(feature = "defmt")]
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {{
        use $crate::defmt::hidden::defmt;
        defmt::debug!($($arg)*);
    }};
}

/// Logs a message at the info level.
#[cfg(feature = "defmt")]
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {{
        use $crate::defmt::hidden::defmt;
        defmt::info!($($arg)*);
    }};
}

/// Logs a message at the warn level.
#[cfg(feature = "defmt")]
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {{
        use $crate::defmt::hidden::defmt;
        defmt::warn!($($arg)*);
    }};
}

/// Logs a message at the error level.
#[cfg(feature = "defmt")]
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        use $crate::defmt::hidden::defmt;
        defmt::error!($($arg)*);
    }};
}

/// No-op log macro.
#[cfg(not(feature = "defmt"))]
#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {{
        let _ = ($($arg)*);
    }};
}

/// No-op log macro.
#[cfg(not(feature = "defmt"))]
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {{
        let _ = ($($arg)*);
    }};
}

/// No-op log macro.
#[cfg(not(feature = "defmt"))]
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {{
        let _ = ($($arg)*);
    }};
}

/// No-op log macro.
#[cfg(not(feature = "defmt"))]
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {{
        let _ = ($($arg)*);
    }};
}

/// No-op log macro.
#[cfg(not(feature = "defmt"))]
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        let _ = ($($arg)*);
    }};
}
