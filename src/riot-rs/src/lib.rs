//! riot-rs
//!
//! This is a meta-package, pulling in the sub-crates of RIOT-rs.
//!
//! # Cargo features
#![doc = document_features::document_features!(feature_label = r#"<span class="stab portability"><code>{feature}</code></span>"#)]
#![no_std]
#![feature(doc_auto_cfg)]

#[cfg(feature = "bench")]
#[doc(inline)]
pub use riot_rs_bench as bench;
#[doc(inline)]
pub use riot_rs_buildinfo as buildinfo;
#[doc(inline)]
pub use riot_rs_debug as debug;
#[doc(inline)]
pub use riot_rs_embassy as embassy;
pub use riot_rs_embassy::{define_peripherals, group_peripherals};
#[cfg(feature = "random")]
#[doc(inline)]
pub use riot_rs_random as random;
#[doc(inline)]
pub use riot_rs_rt as rt;
#[cfg(feature = "threading")]
#[doc(inline)]
pub use riot_rs_threads as thread;

// Attribute macros
pub use riot_rs_macros::config;
pub use riot_rs_macros::spawner;
pub use riot_rs_macros::task;
#[cfg(any(feature = "threading", doc))]
pub use riot_rs_macros::thread;

// These are used by proc-macros we provide
pub use linkme;
pub use static_cell;

// ensure this gets linked
use riot_rs_boards as _;
