//! Synchronous channel implementation for sending data between threads.

use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::mem::MaybeUninit;

use crate::threadlist::ThreadList;
use crate::ThreadState;
use critical_section::with;

enum ChannelState {
    Idle,
    SendersWaiting(ThreadList),
    ReceiversWaiting(ThreadList),
}

/// Blocking channel for sending data between threads.
pub struct Channel<T> {
    state: UnsafeCell<ChannelState>,
    phantom: core::marker::PhantomData<T>,
}

unsafe impl<T> Sync for Channel<T> {}

impl<T: Copy + Send> Channel<T> {
    pub const fn new() -> Self {
        Channel {
            state: UnsafeCell::new(ChannelState::Idle),
            phantom: PhantomData,
        }
    }

    /// Send on the channel (blocking).
    ///
    /// If there is no receiver waiting yet, the current thread is suspended
    /// until a receiver is ready.
    ///
    /// # Panics
    ///
    /// Panics if this is called outside of a thread context.
    pub fn send(&self, something: &T) {
        with(|cs| {
            let state = unsafe { &mut *self.state.get() };
            match state {
                ChannelState::Idle => {
                    let mut waiters = ThreadList::new();
                    waiters.put_current(
                        cs,
                        crate::ThreadState::ChannelTxBlocked(something as *const T as usize),
                    );
                    *state = ChannelState::SendersWaiting(waiters);
                }
                ChannelState::ReceiversWaiting(waiters) => {
                    if let Some((_, head_state)) = waiters.pop(cs) {
                        if waiters.is_empty(cs) {
                            *state = ChannelState::Idle;
                        }
                        if let ThreadState::ChannelRxBlocked(ptr) = head_state {
                            // copy over `something`
                            unsafe { (ptr as *mut T).write(*something) };
                        } else {
                            unreachable!("unexpected thread state");
                        }
                    } else {
                        unreachable!("unexpected empty thread list");
                    }
                }
                ChannelState::SendersWaiting(waiters) => {
                    waiters.put_current(
                        cs,
                        crate::ThreadState::ChannelTxBlocked(self as *const _ as usize),
                    );
                }
            }
        })
    }

    /// Try to send on the channel (non-blocking).
    ///
    /// Returns `true` if a receiver was waiting and received
    /// the data, `false` otherwise.
    pub fn try_send(&self, something: &T) -> bool {
        with(|cs| {
            let state = unsafe { &mut *self.state.get() };
            match state {
                ChannelState::ReceiversWaiting(waiters) => {
                    if let Some((_, head_state)) = waiters.pop(cs) {
                        if waiters.is_empty(cs) {
                            *state = ChannelState::Idle;
                        }
                        if let ThreadState::ChannelRxBlocked(ptr) = head_state {
                            // copy over `something`
                            unsafe { (ptr as *mut T).write(*something) };
                        } else {
                            unreachable!("unexpected thread state");
                        }
                    } else {
                        unreachable!("unexpected empty thread list");
                    }
                    true
                }
                _ => false,
            }
        })
    }

    /// Receive on the channel (blocking).
    ///
    /// If there is no sender waiting yet, the current thread is suspended
    /// until a sender is ready.
    ///
    /// # Panics
    ///
    /// Panics if this is called outside of a thread context.
    pub fn recv(&self) -> T {
        let mut res: MaybeUninit<T> = MaybeUninit::uninit();

        with(|cs| {
            let state = unsafe { &mut *self.state.get() };
            let ptr = res.as_mut_ptr();
            match state {
                ChannelState::Idle => {
                    let mut waiters = ThreadList::new();
                    waiters.put_current(cs, crate::ThreadState::ChannelRxBlocked(ptr as usize));
                    *state = ChannelState::ReceiversWaiting(waiters);
                }
                ChannelState::ReceiversWaiting(waiters) => {
                    waiters.put_current(cs, crate::ThreadState::ChannelRxBlocked(ptr as usize));
                    // sender will copy message
                }
                ChannelState::SendersWaiting(waiters) => {
                    if let Some((_, head_state)) = waiters.pop(cs) {
                        if waiters.is_empty(cs) {
                            *state = ChannelState::Idle;
                        }
                        if let ThreadState::ChannelTxBlocked(other_ptr) = head_state {
                            // copy over `something`
                            unsafe { ptr.write(*(other_ptr as *const T)) };
                        } else {
                            unreachable!("unexpected thread state");
                        }
                    } else {
                        unreachable!("unexpected empty thread list");
                    }
                }
            }
        });

        // ensure the compiler honors what happened to memory while the thread
        // was scheduled away.
        core::sync::atomic::fence(core::sync::atomic::Ordering::Acquire);

        unsafe { res.assume_init() }
    }

    /// Try to send on the channel (non-blocking).
    ///
    /// Returns `Some` data if a sender was waiting and the
    /// data could be received, `None` otherwise.
    pub fn try_recv(&self) -> Option<T> {
        let mut res: MaybeUninit<T> = MaybeUninit::uninit();
        let have_received = with(|cs| {
            let state = unsafe { &mut *self.state.get() };
            match state {
                ChannelState::SendersWaiting(waiters) => {
                    let ptr = res.as_mut_ptr();
                    if let Some((_, head_state)) = waiters.pop(cs) {
                        if waiters.is_empty(cs) {
                            *state = ChannelState::Idle;
                        }
                        if let ThreadState::ChannelTxBlocked(other_ptr) = head_state {
                            // copy over `something`
                            unsafe { ptr.write(*(other_ptr as *const T)) };
                        } else {
                            unreachable!("unexpected thread state");
                        }
                        true
                    } else {
                        unreachable!("unexpected empty thread list");
                    }
                }
                _ => false,
            }
        });

        if have_received {
            core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::Acquire);
            Some(unsafe { res.assume_init() })
        } else {
            None
        }
    }
}

impl<T: Copy + Send> Default for Channel<T> {
    fn default() -> Self {
        Self::new()
    }
}
