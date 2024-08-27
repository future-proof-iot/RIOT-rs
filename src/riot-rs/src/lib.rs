//! RIOT-rs is an operating system for secure, memory-safe, low-power Internet of Things (IoT).
//!
//! See the [README](https://github.com/future-proof-iot/RIOT-rs) for more details.
//!
//! # Examples
//!
//! Application examples can be found in the [`examples` directory](https://github.com/future-proof-iot/RIOT-rs/tree/main/examples).
//!
//! # Cargo features
#![doc = document_features::document_features!(feature_label = r#"<span class="stab portability"><code>{feature}</code></span>"#)]
#![no_std]
#![feature(doc_auto_cfg)]

pub mod buildinfo;

#[cfg(feature = "bench")]
#[doc(inline)]
pub use riot_rs_bench as bench;
#[doc(inline)]
pub use riot_rs_debug as debug;
pub use riot_rs_embassy::*;
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
