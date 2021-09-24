//! A lock is a synchronization primitive that is not owned by a particular
//! thread when locked.
//!
//! When a Lock is unlocked, it can be locked using its `acquire()` method.
//!
//! When a Lock is locked, a call to `acquire()` will be blocked until another thread
//! or ISR unlocks the lock. A call to `release()` will unlock the Lock, unless
//! another thread is blocking on `acquire()`. In that case, the Lock remains
//! locked, and one other thread will resume (its call to `acquire()` will return).
//!

use crate::thread::{Thread, ThreadList, ThreadState};
use core::cell::UnsafeCell;
use cortex_m::interrupt::{self, CriticalSection};

pub struct Lock {
    state: interrupt::Mutex<UnsafeCell<LockState>>,
}

enum LockState {
    Unlocked,
    Locked(ThreadList),
}

impl Lock {
    /// Create a new, unlocked Lock.
    pub const fn new() -> Lock {
        Lock {
            state: interrupt::Mutex::new(UnsafeCell::new(LockState::Unlocked)),
        }
    }

    /// Create a new, locked Lock.
    pub const fn new_locked() -> Lock {
        Lock {
            state: interrupt::Mutex::new(UnsafeCell::new(LockState::Locked(ThreadList::new()))),
        }
    }

    /// Check the state of a Lock.
    ///
    /// returns `true` if the Lock is locked, `false` otherwise.
    ///
    /// Note: This method is safe to be called from ISR context.
    pub fn is_locked(&self) -> bool {
        interrupt::free(|cs| match self.get_state_mut(cs) {
            LockState::Unlocked => true,
            _ => false,
        })
    }

    /// Acquires a Lock. Blocks if it is already locked.
    ///
    /// If the Lock was previously unlocked, this method will "lock" it and return.
    /// If the Lock was previously locked, this method will block until another thread or ISR
    /// calls the Lock `release()` or `try_release()` method. This function will
    /// return after that, with the Lock being locked.
    ///
    /// Note: _Not allowed to be called from ISR context!_
    pub fn acquire(&self) {
        interrupt::free(|cs| {
            let state = &mut self.get_state_mut(cs);
            if let LockState::Locked(list) = state {
                Thread::current().wait_on(list, ThreadState::Paused);
            // other thread has popped us off the list and reset our thread state
            } else {
                **state = LockState::Locked(ThreadList::new());
            }
        });
    }

    /// Acquires a Lock it is unlocked.
    ///
    /// If the Lock was previously unlocked, this method will "lock" it and return true.
    /// If the Lock was previously locked, it remains locked, and the function returns false.
    ///
    /// Note: This method is safe to be called from ISR context.
    pub fn try_acquire(&self) -> bool {
        return interrupt::free(|cs| {
            let state = &mut self.get_state_mut(cs);
            if let LockState::Unlocked = state {
                **state = LockState::Locked(ThreadList::new());
                true
            } else {
                false
            }
        });
    }

    /// Check if current thread is in this lock's waiting list. Remove if it is.
    ///
    /// This function is supposed to be used for implementing acquire() with
    /// timeout.
    ///
    /// which is unfortunately an O(n) operation if it is.
    ///
    /// Returns `true` if the thread was removed from the list, `false` otherwise.
    ///
    /// Note: _Not allowed to be called from ISR context!_
    pub(crate) fn fix_cancelled_acquire(&self, cs: &CriticalSection) -> bool {
        let list_entry = Thread::current().list_entry;
        if list_entry.is_linked() {
            let state = &mut self.get_state_mut(cs);
            if let LockState::Locked(list) = state {
                // if this thread was removed, the lock acquire was cancelled.
                // in that case, return false.
                list.remove(Thread::current())
            } else {
                // shouldn't happen
                false
            }
        } else {
            false
        }
    }

    /// Release a Lock.
    ///
    /// If the Lock was previously unlocked, will return.
    /// If the Lock was previously locked, if no other thread is waiting on it,
    /// the Lock will be unlocked.
    /// If the Lock was previously locked and there were thread(s) waiting on it,
    /// the first waiting thread will be woken, and the Lock remains locked.
    ///
    /// Note: This method is safe to be called from ISR context.
    pub fn release(&self) {
        interrupt::free(|cs| {
            let state = &mut self.get_state_mut(cs);
            if let LockState::Locked(list) = state {
                if let Some(waiting_thread) = list.lpop() {
                    // Lock was locked, there's a waiting thread.
                    waiting_thread.set_state(ThreadState::Running);
                    if waiting_thread.prio > Thread::current().prio {
                        Thread::yield_higher();
                    }
                } else {
                    // Lock was locked, there was no waiting thread.
                    **state = LockState::Unlocked;
                }
            } else {
                // Lock was unlocked
            }
        });
    }

    fn get_state_mut(&self, cs: &interrupt::CriticalSection) -> &mut LockState {
        unsafe { &mut *self.state.borrow(cs).get() }
    }
}
