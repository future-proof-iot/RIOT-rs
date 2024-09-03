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

// Used by macros provided by this crate.
pub use embassy_futures;
pub use embassy_time;

pub use embedded_hal_async;

/// Represents a frequency expressed in kilohertz.
// Do not implement From<u32>, we want to enforce using the constructor for instantiation.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[expect(non_camel_case_types)]
pub struct kHz(pub u32);

impl kHz {
    /// Returns the frequency in kilohertz.
    pub const fn khz(self) -> u32 {
        self.0
    }
}
