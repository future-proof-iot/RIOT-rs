#![deny(missing_docs)]
#![deny(clippy::pedantic)]

use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
};

use ariel_os_runqueue::{RunqueueId, ThreadId};
use critical_section::CriticalSection;

use crate::{thread::ThreadState, threadlist::ThreadList, SCHEDULER};

/// A basic mutex with priority inheritance.
pub struct Mutex<T> {
    state: UnsafeCell<LockState>,
    inner: UnsafeCell<T>,
}

/// State of a [`Mutex`].
enum LockState {
    Unlocked,
    Locked {
        //. The current owner of the lock.
        owner_id: ThreadId,
        /// The original priority of the current owner (without priority inheritance).
        owner_prio: RunqueueId,
        //. Waiters for the mutex.
        waiters: ThreadList,
    },
}

impl LockState {
    /// Returns a [`LockState::Locked`] with the current thread as the owner
    /// and an empty waitlist.
    ///
    /// # Panics
    ///
    /// Panics if called outside of a thread context.
    fn locked_with_current(cs: CriticalSection) -> Self {
        let (owner_id, owner_prio) = SCHEDULER.with_mut_cs(cs, |mut scheduler| {
            let current = scheduler
                .current()
                .expect("Function should be called inside a thread context.");
            (current.pid, current.prio)
        });
        LockState::Locked {
            waiters: ThreadList::new(),
            owner_id,
            owner_prio,
        }
    }
}

impl<T> Mutex<T> {
    /// Creates a new **unlocked** [`Mutex`].
    pub const fn new(value: T) -> Self {
        Self {
            state: UnsafeCell::new(LockState::Unlocked),
            inner: UnsafeCell::new(value),
        }
    }
}

impl<T> Mutex<T> {
    /// Returns whether the mutex is locked.
    pub fn is_locked(&self) -> bool {
        critical_section::with(|_| {
            let state = unsafe { &*self.state.get() };
            !matches!(state, LockState::Unlocked)
        })
    }

    /// Acquires a mutex, blocking the current thread until it is able to do so.
    ///
    /// If the mutex was unlocked, it will be locked and a [`MutexGuard`] is returned.
    /// If the mutex is locked, this function will block the current thread until the mutex gets
    /// unlocked elsewhere.
    ///
    /// If the current owner of the mutex has a lower priority than the current thread, it will inherit
    /// the waiting thread's priority.
    /// The priority is reset once the mutex is released. This means that a **user can not change a thread's
    /// priority while it holds the lock**, because it will be changed back after release!
    ///
    /// # Panics
    ///
    /// Panics if called outside of a thread context.
    pub fn lock(&self) -> MutexGuard<T> {
        critical_section::with(|cs| {
            // SAFETY: access to the state only happens in critical sections, so it's always unique.
            let state = unsafe { &mut *self.state.get() };
            match state {
                LockState::Unlocked => {
                    *state = LockState::locked_with_current(cs);
                }
                LockState::Locked {
                    waiters,
                    owner_id,
                    owner_prio,
                } => {
                    // Insert thread in waitlist, which also triggers the scheduler.
                    match waiters.put_current(cs, ThreadState::LockBlocked) {
                        // `Some` when the inserted thread is the highest priority
                        // thread in the waitlist.
                        Some(waiter_prio) if waiter_prio > *owner_prio => {
                            // Current mutex owner inherits the priority.
                            SCHEDULER.with_mut_cs(cs, |mut scheduler| {
                                scheduler.set_priority(*owner_id, waiter_prio);
                            });
                        }
                        _ => {}
                    }
                    // Context switch happens here as soon as we leave the critical section.
                }
            }
        });
        // Mutex was either directly acquired because it was unlocked, or the current thread was entered
        // to the waitlist. In the latter case, it only continues running here after it was popped again
        // from the waitlist and the thread acquired the mutex.

        MutexGuard { mutex: self }
    }

    /// Attempts to acquire this lock, in a non-blocking fashion.
    ///
    /// If the mutex was unlocked, it will be locked and a [`MutexGuard`] is returned.
    /// If the mutex was locked `None` is returned.
    pub fn try_lock(&self) -> Option<MutexGuard<T>> {
        critical_section::with(|cs| {
            // SAFETY: access to the state only happens in critical sections, so it's always unique.
            let state = unsafe { &mut *self.state.get() };
            if let LockState::Unlocked = *state {
                *state = LockState::locked_with_current(cs);
                Some(MutexGuard { mutex: self })
            } else {
                None
            }
        })
    }

    /// Releases the mutex.
    ///
    /// If there are waiters, the first waiter will be woken up.
    fn release(&self) {
        critical_section::with(|cs| {
            // SAFETY: access to the state only happens in critical sections, so it's always unique.
            let state = unsafe { &mut *self.state.get() };
            if let LockState::Locked {
                waiters,
                owner_id,
                owner_prio,
            } = state
            {
                // Reset original priority of owner.
                SCHEDULER.with_mut_cs(cs, |mut scheduler| {
                    scheduler.set_priority(*owner_id, *owner_prio);
                });
                // Pop next thread from waitlist so that it can acquire the mutex.
                if let Some((pid, _)) = waiters.pop(cs) {
                    SCHEDULER.with_mut_cs(cs, |scheduler| {
                        *owner_id = pid;
                        *owner_prio = scheduler.get_unchecked(pid).prio;
                    });
                } else {
                    // Unlock if waitlist was empty.
                    *state = LockState::Unlocked;
                }
            }
        });
    }
}

unsafe impl<T> Sync for Mutex<T> {}

/// Grants access to the [`Mutex`] inner data.
///
/// Dropping the [`MutexGuard`] will unlock the [`Mutex`];
#[expect(
    clippy::module_name_repetitions,
    reason = "consistency with std and embassy-sync"
)]
pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
}

impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: MutexGuard always has unique access.
        unsafe { &*self.mutex.inner.get() }
    }
}

impl<'a, T> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: MutexGuard always has unique access.
        unsafe { &mut *self.mutex.inner.get() }
    }
}

impl<'a, T> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        // Unlock the mutex when the guard is dropped.
        self.mutex.release();
    }
}

impl<T> !Send for MutexGuard<'_, T> {}

unsafe impl<T: Sync> Sync for MutexGuard<'_, T> {}
