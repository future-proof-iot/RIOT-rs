//! A lock is a synchronization primitive that is not owned by a particular
//! thread when locked.

use crate::thread::{Thread, ThreadList, ThreadState};
use core::cell::UnsafeCell;
use cortex_m::interrupt;

pub struct Lock {
    state: interrupt::Mutex<UnsafeCell<LockState>>,
}

pub enum LockState {
    Unlocked,
    Locked(ThreadList),
}

impl Lock {
    pub const fn new() -> Lock {
        Lock {
            state: interrupt::Mutex::new(UnsafeCell::new(LockState::Unlocked)),
        }
    }

    // pub const fn new_locked() -> Lock {
    //     Lock {
    //         state: interrupt::Mutex::new(UnsafeCell::new(LockState::Locked(ThreadList::new()))),
    //     }
    // }

    pub fn is_locked(&self) -> bool {
        interrupt::free(|cs| match self.get_state_mut(cs) {
            LockState::Unlocked => true,
            _ => false,
        })
    }

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

    pub fn release(&self) {
        interrupt::free(|cs| {
            let state = &mut self.get_state_mut(cs);
            if let LockState::Locked(list) = state {
                if let Some(waiting_thread) = list.lpop() {
                    waiting_thread.set_state(ThreadState::Running);
                    if waiting_thread.prio > Thread::current().prio {
                        Thread::yield_higher();
                    }
                } else {
                    **state = LockState::Unlocked;
                }
            } else {
                // what now. panic?
            }
        });
    }

    fn get_state_mut(&self, cs: &interrupt::CriticalSection) -> &mut LockState {
        unsafe { &mut *self.state.borrow(cs).get() }
    }
}
