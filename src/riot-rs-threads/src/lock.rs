//! This module provides a Lock implementation.
use core::cell::UnsafeCell;

use super::threadlist::ThreadList;
use super::ThreadState;

/// A basic locking object.
///
/// A `Lock` behaves like a Mutex, but carries no data.
/// This is supposed to be used to implement other locking primitives.
pub struct Lock {
    state: UnsafeCell<LockState>,
}

unsafe impl Sync for Lock {}

enum LockState {
    Unlocked,
    Locked(ThreadList),
}

impl Lock {
    /// Creates new **unlocked** Lock
    #[must_use]
    pub const fn new() -> Self {
        Self {
            state: UnsafeCell::new(LockState::Unlocked),
        }
    }

    /// Creates new **locked** Lock
    #[must_use]
    pub const fn new_locked() -> Self {
        Self {
            state: UnsafeCell::new(LockState::Locked(ThreadList::new())),
        }
    }

    /// Returns the current lock state
    ///
    /// true if locked, false otherwise
    pub fn is_locked(&self) -> bool {
        critical_section::with(|_| {
            let state = unsafe { &*self.state.get() };
            !matches!(state, LockState::Unlocked)
        })
    }

    /// Get this lock (blocking)
    ///
    /// If the lock was unlocked, it will be locked and the function returns.
    /// If the lock was locked, this function will block the current thread until the lock gets
    /// unlocked elsewhere.
    ///
    /// **NOTE**: must not be called outside thread context!
    pub fn acquire(&self) {
        critical_section::with(|cs| {
            let state = unsafe { &mut *self.state.get() };
            match state {
                LockState::Unlocked => *state = LockState::Locked(ThreadList::new()),
                LockState::Locked(waiters) => {
                    waiters.put_current(cs, ThreadState::LockBlocked);
                }
            }
        });
    }

    /// Get the lock (non-blocking)
    ///
    /// If the lock was unlocked, it will be locked and the function returns true.
    /// If the lock was locked, the function returns false
    pub fn try_acquire(&self) -> bool {
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

    /// Releases the lock.
    ///
    /// If the lock was locked, and there were waiters, the first waiter will be
    /// woken up.
    /// If the lock was locked and there were no waiters, the lock will be unlocked.
    /// If the lock was not locked, the function just returns.
    pub fn release(&self) {
        critical_section::with(|cs| {
            let state = unsafe { &mut *self.state.get() };
            match state {
                LockState::Unlocked => {}
                LockState::Locked(waiters) => {
                    if waiters.pop(cs).is_none() {
                        *state = LockState::Unlocked;
                    }
                }
            }
        });
    }
}

impl Default for Lock {
    fn default() -> Self {
        Self::new()
    }
}
