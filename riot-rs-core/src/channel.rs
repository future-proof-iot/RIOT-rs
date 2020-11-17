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

pub enum TryRecvError {
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

    fn wait_rx(&mut self) -> T {
        #[cfg(test)]
        println!("wait_rx()");
        if let ChannelState::EmptyWithRxBlocked(waiters) = &mut self.state {
            let mut res = MaybeUninit::uninit();
            Thread::current().wait_on(
                waiters,
                ThreadState::ChannelRxBlocked(res.as_mut_ptr() as usize),
            );
            Thread::yield_higher();
            Thread::isr_enable_disable();
            unsafe { res.assume_init() }
        } else {
            unreachable!();
        }
    }

    pub fn recv(&mut self) -> T {
        interrupt::free(|_| match &mut self.state {
            ChannelState::Idle => {
                #[cfg(test)]
                println!("recv idle()");
                self.state = ChannelState::EmptyWithRxBlocked(ThreadList::new());
                self.wait_rx()
            }
            ChannelState::MessagesAvailable => {
                #[cfg(test)]
                println!("recv avail()");
                // unwrap always succeeds
                let res = self.rb.get().unwrap();
                if self.rb.is_empty() {
                    self.state = ChannelState::Idle;
                }
                res
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
                        panic!("unexpected state in Channel::recv()");
                    }
                    msg
                } else {
                    if let ThreadState::ChannelTxBlocked(ptr) = sender.state {
                        let res = unsafe { *(ptr as *mut T) };
                        sender.set_state(ThreadState::Running);
                        if senders.is_empty() {
                            self.state = ChannelState::MessagesAvailable;
                        }
                        res
                    } else {
                        panic!("unexpected state in Channel::recv()");
                    }
                };
                Thread::yield_higher();
                Thread::isr_enable_disable();
                res
            }
            ChannelState::EmptyWithRxBlocked(_) => {
                #[cfg(test)]
                println!("recv empty with waiters()");
                self.wait_rx()
            }
        })
    }

    fn wait_tx(&mut self, msg: T) {
        #[cfg(test)]
        println!("wait_tx()");
        if let ChannelState::FullWithTxBlocked(waiters) = &mut self.state {
            Thread::current().wait_on(
                waiters,
                ThreadState::ChannelTxBlocked(&msg as *const T as usize),
            );
            Thread::yield_higher();
        } else {
            unreachable!();
        }
    }

    pub fn send(&mut self, msg: T) {
        interrupt::free(|_| match &mut self.state {
            ChannelState::Idle | ChannelState::MessagesAvailable => {
                if self.rb.put(msg) {
                    #[cfg(test)]
                    println!("send idle avail");
                    self.state = ChannelState::MessagesAvailable;
                } else {
                    #[cfg(test)]
                    println!("send idle full");
                    self.state = ChannelState::FullWithTxBlocked(ThreadList::new());
                    self.wait_tx(msg)
                }
            }
            ChannelState::FullWithTxBlocked(_) => self.wait_tx(msg),
            ChannelState::EmptyWithRxBlocked(waiters) => {
                #[cfg(test)]
                println!("tx empty with waiters()");
                let waiter = waiters.lpop().unwrap();
                if let ThreadState::ChannelRxBlocked(ptr) = waiter.state {
                    #[cfg(test)]
                    println!("copying message");
                    unsafe { *(ptr as *mut T) = msg };
                    waiter.set_state(ThreadState::Running);
                    Thread::yield_higher();
                } else {
                    unreachable!();
                }
            }
        })
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

    unsafe {
        Thread::create(
            &mut STACK,
            func,
            &channel as *const Channel<u32> as usize,
            1,
            CreateFlags::empty(),
        );
    }

    println!("recv()");
    assert_eq!(channel.recv(), 0u32);
    assert_eq!(channel.recv(), 1u32);
    assert_eq!(channel.recv(), 2u32);
    assert_eq!(channel.recv(), 3u32);
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
}
