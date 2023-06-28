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

pub struct Channel<T: Copy + Send> {
    state: UnsafeCell<ChannelState>,
    phantom: core::marker::PhantomData<T>,
}

unsafe impl<T: Copy + Send> Sync for Channel<T> {}

impl<T: Copy + Send> Channel<T> {
    pub const fn new() -> Self {
        Channel {
            state: UnsafeCell::new(ChannelState::Idle),
            phantom: PhantomData,
        }
    }

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
                            unsafe { *(ptr as *mut T) = *something };
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
                            unsafe { *(ptr as *mut T) = *something };
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

    pub fn recv(&self) -> T {
        let mut res = MaybeUninit::uninit();
        with(|cs| {
            let state = unsafe { &mut *self.state.get() };
            match state {
                ChannelState::Idle => {
                    let mut waiters = ThreadList::new();
                    waiters.put_current(
                        cs,
                        crate::ThreadState::ChannelRxBlocked(res.as_ptr() as usize),
                    );
                    *state = ChannelState::ReceiversWaiting(waiters);
                }
                ChannelState::ReceiversWaiting(waiters) => {
                    waiters.put_current(
                        cs,
                        crate::ThreadState::ChannelRxBlocked(res.as_ptr() as usize),
                    );
                    // sender will copy message
                }
                ChannelState::SendersWaiting(waiters) => {
                    if let Some((_, head_state)) = waiters.pop(cs) {
                        if waiters.is_empty(cs) {
                            *state = ChannelState::Idle;
                        }
                        if let ThreadState::ChannelTxBlocked(ptr) = head_state {
                            // copy over `something`
                            unsafe { res.write(*(ptr as *const T)) };
                        } else {
                            unreachable!("unexpected thread state");
                        }
                    } else {
                        unreachable!("unexpected empty thread list");
                    }
                }
            }
        });

        unsafe { res.assume_init() }
    }

    pub fn try_recv(&self) -> Option<T> {
        let mut res = MaybeUninit::uninit();
        let have_received = with(|cs| {
            let state = unsafe { &mut *self.state.get() };
            match state {
                ChannelState::SendersWaiting(waiters) => {
                    if let Some((_, head_state)) = waiters.pop(cs) {
                        if waiters.is_empty(cs) {
                            *state = ChannelState::Idle;
                        }
                        if let ThreadState::ChannelTxBlocked(ptr) = head_state {
                            // copy over `something`
                            unsafe { res.write(*(ptr as *const T)) };
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
            unsafe { Some(res.assume_init()) }
        } else {
            None
        }
    }
}
