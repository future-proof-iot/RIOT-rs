//! This module provides an event that can be waited for.

#![deny(missing_docs)]
#![deny(clippy::pedantic)]

use core::cell::UnsafeCell;

use crate::{threadlist::ThreadList, ThreadState};

/// An [`Event`], allowing to notify multiple threads that some event has happened.
///
/// An [`Event`] manages an internal flag that can be set to true with the [`Self::set()`] method and reset
/// to false with the [`Self::clear()`] method. The [`Self::wait()`] method blocks until the flag is set to true. The
/// flag is set to false initially.
pub struct Event {
    state: UnsafeCell<LockState>,
}

unsafe impl Sync for Event {}

#[derive(Debug)]
enum LockState {
    Unlocked,
    Locked(ThreadList),
}

impl Default for LockState {
    fn default() -> Self {
        LockState::Locked(ThreadList::default())
    }
}

impl Event {
    /// Creates a new **unset** [`Event`].
    #[must_use]
    pub const fn new() -> Self {
        Self {
            state: UnsafeCell::new(LockState::Locked(ThreadList::new())),
        }
    }

    /// Creates a new **set** [`Event`].
    #[must_use]
    pub const fn new_set() -> Self {
        Self {
            state: UnsafeCell::new(LockState::Unlocked),
        }
    }

    /// Returns whether the [`Event`] is set.
    pub fn is_set(&self) -> bool {
        critical_section::with(|_| {
            let state = unsafe { &*self.state.get() };
            matches!(state, LockState::Unlocked)
        })
    }

    /// Waits for this [`Event`] to be set (blocking).
    ///
    /// If the event was set, this function returns directly.
    /// If the event was unset, this function will block the current thread until
    /// the event gets set elsewhere.
    ///
    /// # Panics
    ///
    /// Panics if this is called outside of a thread context.
    pub fn wait(&self) {
        critical_section::with(|cs| {
            let state = unsafe { &mut *self.state.get() };
            match state {
                LockState::Unlocked => {}
                LockState::Locked(waiters) => {
                    waiters.put_current(cs, ThreadState::LockBlocked);
                }
            }
        });
    }

    /// Clears the event (non-blocking).
    ///
    /// If the event was set, it will be cleared and the function returns true.
    /// If the event was unset, the function returns false.
    pub fn clear(&self) -> bool {
        critical_section::with(|_| {
            let state = unsafe { &mut *self.state.get() };
            match state {
                LockState::Unlocked => {
                    *state = LockState::Locked(ThreadList::new());
                    true
                }
                LockState::Locked(_) => false,
            }
        })
    }

    /// Sets the event.
    ///
    /// If the event was unset, and there were waiters, all waiters will be
    /// woken up.
    /// If the event was already set, the function just returns.
    pub fn set(&self) {
        critical_section::with(|cs| {
            let state = unsafe { &mut *self.state.get() };
            match state {
                LockState::Unlocked => {}
                LockState::Locked(waiters) => {
                    // unlock all waiters
                    // TODO (opt): A to-be-written `pop_all()` might save cycles.
                    while waiters.pop(cs).is_some() {}
                    *state = LockState::Unlocked;
                }
            }
        });
    }
}

impl Default for Event {
    fn default() -> Self {
        Self::new()
    }
}
