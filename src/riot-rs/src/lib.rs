//! riot-rs
//!
//! This is a meta-package, pulling in the sub-crates of RIOT-rs.

#![no_std]

// silence warning, using this imports RIOT-c's vector tables
#[allow(unused_imports)]
pub use riot_build as _;

pub use riot_rs_buildinfo as buildinfo;
pub use riot_rs_core::thread;
pub use riot_rs_rt as rt;

#[cfg(feature = "embassy")]
pub use riot_rs_embassy as embassy;
