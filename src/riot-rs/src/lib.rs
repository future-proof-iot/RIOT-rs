//! riot-rs
//!
//! This is a meta-package, pulling in the sub-crates of RIOT-rs.

#![no_std]

pub use riot_build as _;
pub use riot_rs_core as core;
pub use riot_rs_rt as rt;
