//! Provides async functionality.
//!
//! This module bundles RIOT-rs async functionality. It also wraps
//! some [Embassy](https://github.com/embassy-rs/embassy) types.

#![deny(missing_docs)]
#![deny(clippy::pedantic)]

use core::cell::OnceCell;

use embassy_sync::blocking_mutex::CriticalSectionMutex;

pub use embassy_executor::{SendSpawner, Spawner};

#[cfg(feature = "executor-thread")]
pub use crate::thread_executor;

pub(crate) static SPAWNER: CriticalSectionMutex<OnceCell<SendSpawner>> =
    CriticalSectionMutex::new(OnceCell::new());

/// Gets a spawner for the system executor.
///
/// # Panics
///
/// Panics when called before the system has finished initializing.
pub fn spawner() -> SendSpawner {
    SPAWNER.lock(|x| *x.get().unwrap())
}

/// Sets what `spawner()` returns.
///
/// May only be called once. Basically only in `super::init_task()`. That's why
/// we get away with ignoring the result.
#[allow(dead_code, reason = "actually used in `crate::init_task()`")]
pub(crate) fn set_spawner(spawner: SendSpawner) {
    let _ = SPAWNER.lock(|x| x.set(spawner));
}
