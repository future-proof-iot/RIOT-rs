//! HAL-agnostic types shared between HALs.

#![no_std]
#![feature(doc_auto_cfg)]
#![cfg_attr(
    not(context = "xtensa"),
    expect(
        stable_features,
        reason = "feature(const_option) is needed for Xtensa toolchain that is held behind"
    )
)]
#![feature(const_option)]
#![deny(clippy::pedantic)]
#![deny(missing_docs)]

pub mod gpio;

#[cfg(context = "cortex-m")]
pub mod executor_swi;

#[cfg(feature = "i2c")]
pub mod i2c;

pub mod identity;

#[cfg(feature = "spi")]
pub mod spi;

pub mod reexports {
    //! Crate re-exports.

    // Used by macros provided by this crate.
    pub use embassy_futures;
    pub use embassy_time;
    pub use embedded_hal_async;
}
