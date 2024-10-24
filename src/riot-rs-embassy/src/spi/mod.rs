//! Provides support for the SPI communication bus.
#![deny(missing_docs)]

#[doc(alias = "master")]
pub mod main;

pub use riot_rs_embassy_common::spi::*;
