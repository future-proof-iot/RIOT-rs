//! Pass non-Send objects around on same executor
//!
//! This module provides `SendCell`, a structure that allows passing around
//! non-Send objects from one async task to another, if they are on the same
//! executor. This is allowed because embassy executors are single threaded.
//! `SendCell` checks for the correct executor *at runtime*.

use embassy_executor::Spawner;

// SAFETY:
// SendCell guarantees at runtime that its content stays on the same embassy
// executor. Those are single threaded, so it is guaranteed that the content
// stays on the same thread.
unsafe impl<T> Send for SendCell<T> {}

/// A cell that allows sending of non-Send types *if they stay on the same executor*.
///
/// This is checked *at runtime*.
#[derive(Debug)]
pub struct SendCell<T> {
    executor_id: usize,
    inner: T,
}

impl<T> SendCell<T> {
    /// Create a new `SendCell`
    ///
    /// The `spawner` argument *must* point to the current executor.
    pub fn new(inner: T, spawner: &Spawner) -> Self {
        Self {
            executor_id: spawner.executor_id(),
            inner,
        }
    }

    /// Get content of this `SendCell`
    ///
    /// The `spawner` argument *must* point to the current executor.
    pub fn get(&self, spawner: &Spawner) -> Option<&T> {
        if spawner.executor_id() == self.executor_id {
            Some(&self.inner)
        } else {
            None
        }
    }
}
