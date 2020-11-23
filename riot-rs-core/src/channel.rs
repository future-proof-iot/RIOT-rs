// multi producer multi receiver channel
//
// TODO: This is a first implementation, built to match RIOT's semantics.
//       It feels too complex.
//

use core::mem::MaybeUninit;

use cortex_m::interrupt;

use crate::thread::{Thread, ThreadList, ThreadState};
use queue::RingBuffer;

#[cfg(test)]
use riot_rs_rt::debug::println;

pub enum ChannelState {
    Idle,
    MessagesAvailable,
    FullWithTxBlocked(ThreadList),
    EmptyWithRxBlocked(ThreadList),
}

pub struct Channel<'a, T>
where
    T: Copy,
{
    rb: RingBuffer<'a, T>,
    state: ChannelState,
}

#[derive(Debug, PartialEq)]
pub enum TryRecvError {
    WouldBlock,
}

#[derive(Debug, PartialEq)]
pub enum TrySendError {
    WouldBlock,
}

impl<'a, T> Channel<'a, T>
where
    T: Copy,
{
    pub const fn new(backing_array: &mut [MaybeUninit<T>]) -> Channel<T> {
        Channel {
            rb: RingBuffer::new(backing_array),
            state: ChannelState::Idle,
        }
    }

    fn try_recv_impl(&mut self) -> Result<T, TryRecvError> {
        match &mut self.state {
            ChannelState::MessagesAvailable => {
                #[cfg(test)]
                println!("recv avail()");

                // unwrap always succeeds
                let res = self.rb.get().unwrap();
                if self.rb.is_empty() {
                    self.state = ChannelState::Idle;
                }
                Ok(res)
            }
            ChannelState::FullWithTxBlocked(senders) => {
                #[cfg(test)]
                println!("recv full with senders()");

                // unwrap always succeeds
                let sender = senders.lpop().unwrap();
                let res = if let Some(msg) = self.rb.get() {
                    if let ThreadState::ChannelTxBlocked(ptr) = sender.state {
                        self.rb.put(unsafe { *(ptr as *mut T) });
                        sender.set_state(ThreadState::Running);
                        if senders.is_empty() {
                            self.state = ChannelState::MessagesAvailable;
                        }
                    } else {
                        unreachable!();
                    }
                    Ok(msg)
                } else {
                    #[cfg(test)]
                    println!("recv direct copy()");
                    if let ThreadState::ChannelTxBlocked(ptr) = sender.state {
                        let res = unsafe { *(ptr as *mut T) };
                        sender.set_state(ThreadState::Running);
                        if senders.is_empty() {
                            self.state = ChannelState::MessagesAvailable;
                        }
                        Ok(res)
                    } else {
                        unreachable!();
                    }
                };
                Thread::yield_higher();
                res
            }
            _ => Err(TryRecvError::WouldBlock),
        }
    }

    pub fn try_recv(&mut self) -> Result<T, TryRecvError> {
        interrupt::free(|_| self.try_recv_impl())
    }

    pub fn recv(&mut self) -> T {
        interrupt::free(|_| {
            if let Ok(res) = self.try_recv_impl() {
                res
            } else {
                if let ChannelState::Idle = self.state {
                    self.state = ChannelState::EmptyWithRxBlocked(ThreadList::new());
                }
                let mut res = MaybeUninit::uninit();
                if let ChannelState::EmptyWithRxBlocked(waiters) = &mut self.state {
                    Thread::current().wait_on(
                        waiters,
                        ThreadState::ChannelRxBlocked(res.as_mut_ptr() as usize),
                    );
                } else {
                    unreachable!();
                }
                Thread::yield_higher();
                Thread::isr_enable_disable();
                unsafe { res.assume_init() }
            }
        })
    }

    pub fn try_send_impl(&mut self, msg: T) -> Result<(), TrySendError> {
        if let ChannelState::EmptyWithRxBlocked(waiters) = &mut self.state {
            #[cfg(test)]
            println!("tx empty with waiters()");
            let waiter = waiters.lpop().unwrap();
            if let ThreadState::ChannelRxBlocked(ptr) = waiter.state {
                #[cfg(test)]
                println!("copying message");
                unsafe { *(ptr as *mut T) = msg };
                waiter.set_state(ThreadState::Running);
                Thread::yield_higher();
                if waiters.is_empty() {
                    self.state = ChannelState::Idle;
                }
            } else {
                unreachable!();
            }
            Ok(())
        } else {
            if self.rb.put(msg) {
                #[cfg(test)]
                println!("try_send() put message ok");
                if let ChannelState::Idle = self.state {
                    #[cfg(test)]
                    println!("setting state to msgavail");
                    self.state = ChannelState::MessagesAvailable;
                }
                Ok(())
            } else {
                #[cfg(test)]
                println!("try_send() wouldblock");
                Err(TrySendError::WouldBlock)
            }
        }
    }

    pub fn try_send(&mut self, msg: T) -> Result<(), TrySendError> {
        interrupt::free(|_| self.try_send_impl(msg))
    }

    pub fn send(&mut self, msg: T) {
        interrupt::free(|_| {
            if self.try_send_impl(msg).is_err() {
                match self.state {
                    ChannelState::FullWithTxBlocked(_) => (),
                    _ => self.state = ChannelState::FullWithTxBlocked(ThreadList::new()),
                }
                if let ChannelState::FullWithTxBlocked(waiters) = &mut self.state {
                    #[cfg(test)]
                    println!("send() waiting");
                    Thread::current().wait_on(
                        waiters,
                        ThreadState::ChannelTxBlocked(&msg as *const T as usize),
                    );
                    Thread::yield_higher();
                } else {
                    unreachable!();
                }
            }
        })
    }

    pub fn capacity(&self) -> usize {
        self.rb.capacity()
    }

    pub fn available(&self) -> usize {
        self.rb.available()
    }
}

pub mod c {
    #![allow(non_camel_case_types)]
    use super::Channel;
    use crate::thread::c::msg_t;
    use core::mem::MaybeUninit;
    use ref_cast::RefCast;

    #[derive(RefCast)]
    #[repr(transparent)]
    pub struct mbox_t(Channel<'static, msg_t>);

    pub const MBOX_T_SIZEOF: usize = 20;
    pub const MBOX_T_ALIGNOF: usize = 4;

    #[test_case]
    fn test_mbox_const_defines() {
        assert_eq!(core::mem::size_of::<mbox_t>(), MBOX_T_SIZEOF);
        assert_eq!(core::mem::align_of::<mbox_t>(), MBOX_T_ALIGNOF);
    }

    #[no_mangle]
    pub unsafe extern "C" fn mbox_init(
        mbox: &'static mut mbox_t,
        queue: &'static mut msg_t,
        queue_size: usize,
    ) {
        let queue: &'static mut MaybeUninit<msg_t> = core::mem::transmute(queue);
        let queue = core::slice::from_raw_parts_mut(queue, queue_size);
        *mbox = mbox_t {
            0: Channel::new(queue),
        };
    }

    #[no_mangle]
    pub unsafe extern "C" fn mbox_put(mbox: &mut mbox_t, msg: &mut msg_t) {
        mbox.0.send(*msg)
    }

    #[no_mangle]
    pub unsafe extern "C" fn mbox_get(mbox: &mut mbox_t, msg: &mut msg_t) {
        *msg = mbox.0.recv()
    }

    #[no_mangle]
    pub unsafe extern "C" fn mbox_try_put(mbox: &mut mbox_t, msg: &mut msg_t) -> bool {
        match mbox.0.try_send(*msg) {
            Ok(()) => true,
            _ => false,
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn mbox_try_get(mbox: &mut mbox_t, msg: &mut msg_t) -> bool {
        match mbox.0.try_recv() {
            Ok(res) => {
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

#[test_case]
fn test_channel_lowprio_sender() {
    use crate::thread::{CreateFlags, Thread};
    use riot_rs_rt::debug::println;

    static mut STACK: [u8; 1024] = [0; 1024];

    fn func(arg: usize) {
        let channel = unsafe { &mut *(arg as *mut Channel<u32>) };
        for i in 0..4u32 {
            channel.send(i);
        }
    }

    let mut array: [MaybeUninit<u32>; 4] = unsafe { MaybeUninit::uninit().assume_init() };
    let mut channel = Channel::new(&mut array);

    assert_eq!(channel.try_recv(), Err(TryRecvError::WouldBlock));

    unsafe {
        Thread::create(
            &mut STACK,
            func,
            &channel as *const Channel<u32> as usize,
            1,
            CreateFlags::empty(),
        );
    }

    assert_eq!(channel.recv(), 0u32);
    assert_eq!(channel.recv(), 1u32);
    assert_eq!(channel.recv(), 2u32);
    assert_eq!(channel.recv(), 3u32);

    assert_eq!(channel.try_recv(), Err(TryRecvError::WouldBlock));
}

#[test_case]
fn test_channel_hiprio_sender() {
    use crate::thread::{CreateFlags, Thread};
    use riot_rs_rt::debug::println;

    static mut STACK: [u8; 1024] = [0; 1024];

    fn func(arg: usize) {
        let channel = unsafe { &mut *(arg as *mut Channel<u32>) };
        for i in 0..8u32 {
            channel.send(i);
        }
    }

    let mut array: [MaybeUninit<u32>; 4] = unsafe { MaybeUninit::uninit().assume_init() };
    let mut channel = Channel::new(&mut array);

    assert_eq!(channel.try_recv(), Err(TryRecvError::WouldBlock));

    unsafe {
        Thread::create(
            &mut STACK,
            func,
            &channel as *const Channel<u32> as usize,
            6,
            CreateFlags::empty(),
        );
    }

    println!("recv()");
    for i in 0..8u32 {
        assert_eq!(channel.recv(), i);
    }

    assert_eq!(channel.try_recv(), Err(TryRecvError::WouldBlock));
}
