//! multi producer multi receiver channel with queue
//!
//! TODO: This is a first implementation, built to match RIOT's semantics.
//!       It feels too complex.
//!

use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use ringbuffer::RingBuffer;

use crate::thread::ThreadList;
use crate::thread::ThreadState;
use critical_section::with;

enum BufferedChannelState {
    Idle,
    SendersWaiting(ThreadList),
    ReceiversWaiting(ThreadList),
}

pub struct BufferedChannel<'a, T: Copy + Send> {
    state: UnsafeCell<BufferedChannelState>,
    rb: UnsafeCell<RingBuffer<'a, T>>,
    phantom: core::marker::PhantomData<T>,
}

unsafe impl<'a, T: Copy + Send> Sync for BufferedChannel<'a, T> {}

impl<'a, T: Copy + Send> BufferedChannel<'a, T> {
    pub const fn new() -> Self {
        BufferedChannel {
            state: UnsafeCell::new(BufferedChannelState::Idle),
            rb: UnsafeCell::new(RingBuffer::new()),
            phantom: PhantomData,
        }
    }

    pub fn new_with(queue: &'a mut [MaybeUninit<T>]) -> Self {
        BufferedChannel {
            state: UnsafeCell::new(BufferedChannelState::Idle),
            rb: UnsafeCell::new(RingBuffer::new_with(queue)),
            phantom: PhantomData,
        }
    }

    pub fn send(&self, something: &T) {
        with(|cs| {
            let state = unsafe { &mut *self.state.get() };
            match state {
                BufferedChannelState::Idle => {
                    let rb = unsafe { &mut *self.rb.get() };
                    if rb.put(*something) {
                        // done
                    } else {
                        let mut waiters = ThreadList::new();
                        waiters.put_current(
                            cs,
                            ThreadState::ChannelTxBlocked(something as *const T as usize),
                        );
                        *state = BufferedChannelState::SendersWaiting(waiters);
                    }
                }
                BufferedChannelState::ReceiversWaiting(waiters) => {
                    if let Some((_, head_state)) = waiters.pop(cs) {
                        if waiters.is_empty(cs) {
                            *state = BufferedChannelState::Idle;
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
                BufferedChannelState::SendersWaiting(waiters) => {
                    waiters
                        .put_current(cs, ThreadState::ChannelTxBlocked(self as *const _ as usize));
                }
            }
        })
    }

    pub fn try_send(&self, something: &T) -> bool {
        return with(|cs| {
            let state = unsafe { &mut *self.state.get() };
            match state {
                BufferedChannelState::Idle => {
                    let rb = unsafe { &mut *self.rb.get() };
                    rb.put(*something)
                }
                BufferedChannelState::ReceiversWaiting(waiters) => {
                    if let Some((_, head_state)) = waiters.pop(cs) {
                        if waiters.is_empty(cs) {
                            *state = BufferedChannelState::Idle;
                        }
                        if let ThreadState::ChannelRxBlocked(ptr) = head_state {
                            // copy over `something`
                            unsafe { *(ptr as *mut T) = *something };
                        } else {
                            unreachable!("unexpected thread state");
                        }
                        true
                    } else {
                        false
                    }
                }
                BufferedChannelState::SendersWaiting(_) => false,
            }
        });
    }

    pub fn recv(&self) -> T {
        let mut res = MaybeUninit::uninit();
        with(|cs| {
            let state = unsafe { &mut *self.state.get() };
            match state {
                BufferedChannelState::Idle => {
                    let rb = unsafe { &mut *self.rb.get() };
                    if let Some(thing) = rb.get() {
                        res.write(thing);
                    } else {
                        let mut waiters = ThreadList::new();
                        waiters
                            .put_current(cs, ThreadState::ChannelRxBlocked(res.as_ptr() as usize));
                        *state = BufferedChannelState::ReceiversWaiting(waiters);
                    }
                }
                BufferedChannelState::ReceiversWaiting(waiters) => {
                    waiters.put_current(cs, ThreadState::ChannelRxBlocked(res.as_ptr() as usize));
                    // sender will copy message
                }
                BufferedChannelState::SendersWaiting(waiters) => {
                    let rb = unsafe { &mut *self.rb.get() };
                    // first we check the queue.
                    // That might fail even with senders waiting, should the queue
                    // be zero-sized.
                    let mut have_msg = false;
                    if let Some(thing) = rb.get() {
                        res.write(thing);
                        have_msg = true;
                    }
                    // either we `have_msg`, or we'll direct-copy a message,
                    // so pop()'ing a waiter (which wakes it up) is fine.
                    if let Some((_, head_state)) = waiters.pop(cs) {
                        if waiters.is_empty(cs) {
                            *state = BufferedChannelState::Idle;
                        }
                        if let ThreadState::ChannelTxBlocked(ptr) = head_state {
                            if have_msg {
                                // directly add the waiter's message to the queue
                                unsafe { rb.put(*(ptr as *const T)) };
                            } else {
                                // copy over `something`
                                unsafe { res.write(*(ptr as *const T)) };
                            }
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
                BufferedChannelState::Idle => {
                    let rb = unsafe { &mut *self.rb.get() };
                    if let Some(thing) = rb.get() {
                        res.write(thing);
                        true
                    } else {
                        false
                    }
                }
                BufferedChannelState::ReceiversWaiting(_) => false,
                BufferedChannelState::SendersWaiting(waiters) => {
                    let rb = unsafe { &mut *self.rb.get() };
                    // first we check the queue.
                    // That might fail even with senders waiting, should the queue
                    // be zero-sized.
                    let mut have_msg = false;
                    if let Some(thing) = rb.get() {
                        res.write(thing);
                        have_msg = true;
                    }
                    // either we `have_msg`, or we'll direct-copy a message,
                    // so pop()'ing a waiter (which wakes it up) is fine.
                    if let Some((_, head_state)) = waiters.pop(cs) {
                        if waiters.is_empty(cs) {
                            *state = BufferedChannelState::Idle;
                        }
                        if let ThreadState::ChannelTxBlocked(ptr) = head_state {
                            if have_msg {
                                // directly add the waiter's message to the queue
                                unsafe { rb.put(*(ptr as *const T)) };
                            } else {
                                // copy over `something`
                                unsafe { res.write(*(ptr as *const T)) };
                            }
                        } else {
                            unreachable!("unexpected thread state");
                        }
                    } else {
                        unreachable!("unexpected empty thread list");
                    }
                    true
                }
            }
        });

        if have_received {
            unsafe { Some(res.assume_init()) }
        } else {
            None
        }
    }
    pub fn capacity(&self) -> usize {
        with(|_| {
            let rb = unsafe { &mut *self.rb.get() };
            rb.capacity()
        })
    }

    pub fn available(&self) -> usize {
        with(|_| {
            let rb = unsafe { &mut *self.rb.get() };
            rb.available()
        })
    }

    pub fn set_backing_array(&mut self, array: Option<&'a mut [MaybeUninit<T>]>) {
        critical_section::with(|_| {
            let state = unsafe { &mut *self.state.get() };
            let rb = unsafe { &mut *self.rb.get() };
            if let BufferedChannelState::Idle = state {
                rb.set_backing_array(array);
            } else {
                panic!("cannot change backing array unless channel is Idle");
            };
        });
    }
}

/// C bindings for BufferedChannel to support RIOT's "mbox" API
///
/// ("msg" is implemented in thread.rs)
pub mod c {
    #![allow(non_camel_case_types)]
    use super::BufferedChannel;
    use crate::c::msg::msg_t;
    use core::mem::MaybeUninit;
    use ref_cast::RefCast;

    #[derive(RefCast)]
    #[repr(transparent)]
    pub struct mbox_t(BufferedChannel<'static, msg_t>);

    pub const MBOX_T_SIZEOF: usize = 20;
    pub const MBOX_T_ALIGNOF: usize = 4;

    // #[test_case]
    // fn test_mbox_const_defines() {
    //     assert_eq!(core::mem::size_of::<mbox_t>(), MBOX_T_SIZEOF);
    //     assert_eq!(core::mem::align_of::<mbox_t>(), MBOX_T_ALIGNOF);
    // }

    #[no_mangle]
    pub unsafe extern "C" fn mbox_init(
        mbox: &'static mut mbox_t,
        queue: &'static mut msg_t,
        queue_size: usize,
    ) {
        let queue: &'static mut MaybeUninit<msg_t> = core::mem::transmute(queue);
        let queue = core::slice::from_raw_parts_mut(queue, queue_size);
        *mbox = mbox_t {
            0: BufferedChannel::new_with(queue),
        };
    }

    #[no_mangle]
    pub unsafe extern "C" fn mbox_put(mbox: &mut mbox_t, msg: &mut msg_t) {
        mbox.0.send(msg)
    }

    #[no_mangle]
    pub unsafe extern "C" fn mbox_get(mbox: &mut mbox_t, msg: &mut msg_t) {
        *msg = mbox.0.recv()
    }

    #[no_mangle]
    pub unsafe extern "C" fn mbox_try_put(mbox: &mut mbox_t, msg: &mut msg_t) -> bool {
        mbox.0.try_send(msg)
    }

    #[no_mangle]
    pub unsafe extern "C" fn mbox_try_get(mbox: &mut mbox_t, msg: &mut msg_t) -> bool {
        match mbox.0.try_recv() {
            Some(res) => {
                *msg = res;
                true
            }
            _ => false,
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn mbox_size(mbox: &mbox_t) -> usize {
        mbox.0.capacity()
    }

    #[no_mangle]
    pub unsafe extern "C" fn mbox_avail(mbox: &mbox_t) -> usize {
        mbox.0.available()
    }
}
