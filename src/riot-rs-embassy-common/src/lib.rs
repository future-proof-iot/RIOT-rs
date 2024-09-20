//! Architecture-agnostic types shared between architectures.

#![no_std]
#![feature(doc_auto_cfg)]
#![deny(clippy::pedantic)]
#![deny(missing_docs)]

pub mod gpio;

#[cfg(context = "cortex-m")]
pub mod executor_swi;

#[cfg(feature = "i2c")]
pub mod i2c;

pub mod reexports {
    //! Crate re-exports.

    // Used by macros provided by this crate.
    pub use embassy_futures;
    pub use embassy_time;
    pub use embedded_hal_async;
}
