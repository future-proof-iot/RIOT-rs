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

pub mod identity;

pub mod reexports {
    //! Crate re-exports.

    // Used by macros provided by this crate.
    pub use embassy_futures;
    pub use embassy_time;
    pub use embedded_hal_async;
}

/// Soft sealing trait, which opts a dependent trait out of RIOT-rs's stability guarantees.
///
/// Traits that depend on [`Sealed`] are supposed to only be implemented by RIOT-rs internally,
/// e.g. for a particular architecture. As RIOT-rs is composed of a group of crates and Rust has no
/// concept of items being private to a family of crates, this can not be enforced.
///
/// The precise evolution strategy depends on the trait that requires [`Sealed`]; it is up to the
/// implementer of this trait to keep track of changes of *all* RIOT-rs traits which that type
/// implements.
pub trait Sealed {}
