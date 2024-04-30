//! Provides on-board benchmarking facilities.

#![cfg_attr(not(test), no_std)]
#![feature(error_in_core)]
#![deny(missing_docs)]

cfg_if::cfg_if! {
    if #[cfg(context = "cortex-m")] {
        mod cortexm;
        use cortexm as bench;
    }
    else if #[cfg(context = "riot-rs")] {
        // When run with laze but the architecture is not supported
        compile_error!("benchmarking is not supported for this architecture");
    } else {
        // Provide a default bench module, for arch-independent tooling
        mod bench {
            use crate::Error;

            /// Benchmarks "time" required to run the provided function.
            ///
            /// Runs the provided function `iterations` times, and returns the mean number of system timer
            /// increments per iteration.
            ///
            /// # Errors
            ///
            /// Returns [`Error::SystemTimerWrapped`] if the system timer counter has wrapped when
            /// benchmarking.
            #[allow(unused_variables)]
            pub fn benchmark<F: Fn()>(iterations: usize, f: F) -> Result<usize, Error> {
                unimplemented!();
            }
        }
    }
}

pub use bench::benchmark;

/// Possible errors happening when benchmarking.
#[derive(Debug)]
pub enum Error {
    /// The system timer wrapped when benchmarking.
    SystemTimerWrapped,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::SystemTimerWrapped => write!(f, "system timer wrapped"),
        }
    }
}

impl core::error::Error for Error {}
