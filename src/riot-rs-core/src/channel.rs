//! multi producer multi receiver channel
//!
//! TODO: This is a first implementation, built to match RIOT's semantics.
//!       It feels too complex.
//!

pub trait Channel<T>
where
    T: Copy,
{
    fn send(&mut self, msg: T);
    fn recv(&mut self) -> T;
    fn try_send(&mut self, msg: T) -> Result<(), TrySendError>;
    fn try_recv(&mut self) -> Result<T, TryRecvError>;
}

#[derive(Debug, PartialEq)]
pub enum TryRecvError {
    WouldBlock,
}

#[derive(Debug, PartialEq)]
pub enum TrySendError {
    WouldBlock,
}

pub use self::buffered::BufferedChannel;
pub use self::sync::SyncChannel;

/// Channel with (optional) buffer
pub mod buffered {
    use critical_section;
    #[cfg(test)]
    use riot_rs_rt::debug::println;

    use core::mem::MaybeUninit;

    use super::{TryRecvError, TrySendError};
    use crate::thread::{Thread, ThreadList, ThreadState};
    use ringbuffer::RingBuffer;

    #[derive(Debug)]
    pub enum BufferedChannelState {
        Idle,
        MessagesAvailable,
        FullWithTxBlocked(ThreadList),
        EmptyWithRxBlocked(ThreadList),
    }

    #[derive(Debug)]
    pub struct BufferedChannel<'a, T>
    where
        T: Copy,
    {
        rb: RingBuffer<'a, T>,
        state: BufferedChannelState,
    }

    impl<'a, T> BufferedChannel<'a, T>
    where
        T: Copy,
    {
        pub const fn new(backing_array: Option<&mut [MaybeUninit<T>]>) -> BufferedChannel<T> {
            BufferedChannel {
                rb: RingBuffer::new(backing_array),
                state: BufferedChannelState::Idle,
            }
        }

        fn try_recv_impl(&mut self) -> Result<T, TryRecvError> {
            match &mut self.state {
                BufferedChannelState::MessagesAvailable => {
                    #[cfg(test)]
                    println!("recv avail()");

                    // unwrap always succeeds
                    let res = self.rb.get().unwrap();
                    if self.rb.is_empty() {
                        self.state = BufferedChannelState::Idle;
                    }
                    Ok(res)
                }
                BufferedChannelState::FullWithTxBlocked(senders) => {
                    #[cfg(test)]
                    println!("recv full with senders()");

                    // unwrap always succeeds
                    let sender = senders.lpop().unwrap();
                    let res = if let Some(msg) = self.rb.get() {
                        if let ThreadState::ChannelTxBlocked(ptr) = sender.state {
                            self.rb.put(unsafe { core::ptr::read(ptr as *const T) });
                            sender.set_state(ThreadState::Running);
                            if senders.is_empty() {
                                self.state = BufferedChannelState::MessagesAvailable;
                            }
                        } else if let ThreadState::ChannelTxReplyBlocked(ptr) = sender.state {
                            self.rb.put(unsafe { core::ptr::read(ptr as *const T) });
                            sender.set_state(ThreadState::ChannelReplyBlocked(ptr));
                            if senders.is_empty() {
                                self.state = BufferedChannelState::MessagesAvailable;
                            }
                        } else {
                            unreachable!();
                        }
                        Ok(msg)
                    } else {
                        #[cfg(test)]
                        println!("recv direct copy()");
                        if let ThreadState::ChannelTxBlocked(ptr) = sender.state {
                            let res = unsafe { core::ptr::read(ptr as *const T) };
                            sender.set_state(ThreadState::Running);
                            if senders.is_empty() {
                                self.state = BufferedChannelState::Idle;
                            }
                            Ok(res)
                        } else if let ThreadState::ChannelTxReplyBlocked(ptr) = sender.state {
                            let res = unsafe { core::ptr::read(ptr as *const T) };
                            sender.set_state(ThreadState::ChannelReplyBlocked(ptr));
                            if senders.is_empty() {
                                self.state = BufferedChannelState::Idle;
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
            critical_section::with(|_| self.try_recv_impl())
        }

        pub fn recv(&mut self) -> T {
            critical_section::with(|_| {
                if let Ok(res) = self.try_recv_impl() {
                    res
                } else {
                    if let BufferedChannelState::Idle = self.state {
                        self.state = BufferedChannelState::EmptyWithRxBlocked(ThreadList::new());
                    }
                    let mut res = MaybeUninit::uninit();
                    if let BufferedChannelState::EmptyWithRxBlocked(waiters) = &mut self.state {
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
            if let BufferedChannelState::EmptyWithRxBlocked(waiters) = &mut self.state {
                #[cfg(test)]
                println!("tx empty with waiters()");
                let waiter = waiters.lpop().unwrap();
                if let ThreadState::ChannelRxBlocked(ptr) = waiter.state {
                    #[cfg(test)]
                    println!("copying message");
                    unsafe { core::ptr::write_volatile(ptr as *mut T, msg) };
                    waiter.set_state(ThreadState::Running);
                    Thread::yield_higher();
                    if waiters.is_empty() {
                        self.state = BufferedChannelState::Idle;
                    }
                } else {
                    unreachable!();
                }
                Ok(())
            } else {
                if self.rb.put(msg) {
                    #[cfg(test)]
                    println!("try_send() put message ok");
                    if let BufferedChannelState::Idle = self.state {
                        #[cfg(test)]
                        println!("setting state to msgavail");
                        self.state = BufferedChannelState::MessagesAvailable;
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
            critical_section::with(|_| self.try_send_impl(msg))
        }

        pub fn send(&mut self, msg: T) {
            let msg_ref: &T = &msg;
            critical_section::with(|_| {
                if self.try_send_impl(msg).is_err() {
                    match self.state {
                        BufferedChannelState::FullWithTxBlocked(_) => (),
                        _ => {
                            self.state = BufferedChannelState::FullWithTxBlocked(ThreadList::new())
                        }
                    }
                    if let BufferedChannelState::FullWithTxBlocked(waiters) = &mut self.state {
                        #[cfg(test)]
                        println!("send() waiting");
                        Thread::current().wait_on(
                            waiters,
                            ThreadState::ChannelTxBlocked(msg_ref as *const T as usize),
                        );
                        Thread::yield_higher();
                    } else {
                        unreachable!();
                    }
                }
            });
            unsafe { core::arch::asm!("/* {0} */", in(reg) msg_ref) };
        }

        pub fn send_reply(&mut self, msg: T, target: crate::thread::Pid) -> T {
            let mut reply = MaybeUninit::<T>::uninit();
            let reply_ptr = reply.as_mut_ptr();
            critical_section::with(|_| {
                if self.try_send_impl(msg).is_err() {
                    match self.state {
                        BufferedChannelState::FullWithTxBlocked(_) => (),
                        _ => {
                            self.state = BufferedChannelState::FullWithTxBlocked(ThreadList::new())
                        }
                    }
                    if let BufferedChannelState::FullWithTxBlocked(waiters) = &mut self.state {
                        #[cfg(test)]
                        println!("send_reply() waiting");
                        unsafe { core::ptr::write(reply_ptr, msg) };
                        Thread::current().wait_on(
                            waiters,
                            ThreadState::ChannelTxReplyBlocked(reply_ptr as usize),
                        );
                        unsafe {
                            Thread::get_mut(target).flag_set(crate::thread::THREAD_FLAG_MSG_WAITING)
                        };
                        Thread::yield_higher();
                    } else {
                        unreachable!();
                    }
                } else {
                    Thread::current()
                        .set_state(ThreadState::ChannelReplyBlocked(reply_ptr as usize));
                    unsafe {
                        Thread::get_mut(target).flag_set(crate::thread::THREAD_FLAG_MSG_WAITING)
                    };
                    Thread::yield_higher();
                }
            });
            unsafe { reply.assume_init() }
        }

        pub fn capacity(&self) -> usize {
            self.rb.capacity()
        }

        pub fn available(&self) -> usize {
            self.rb.available()
        }

        pub fn set_backing_array(&mut self, array: Option<&'a mut [T]>) {
            critical_section::with(|_| {
                if let BufferedChannelState::Idle = self.state {
                    self.rb.set_backing_array(array);
                } else {
                    panic!("cannot change backing array unless channel is Idle");
                };
            });
        }
    }

    impl<'a, T> super::Channel<T> for BufferedChannel<'a, T>
    where
        T: Copy,
    {
        fn send(&mut self, msg: T) {
            BufferedChannel::send(self, msg)
        }
        fn try_send(&mut self, msg: T) -> Result<(), TrySendError> {
            BufferedChannel::try_send(self, msg)
        }
        fn recv(&mut self) -> T {
            BufferedChannel::recv(self)
        }
        fn try_recv(&mut self) -> Result<T, TryRecvError> {
            BufferedChannel::try_recv(self)
        }
    }
}

/// channel with out buffer
pub mod sync {
    #[cfg(test)]
    use riot_rs_rt::debug::println;

    use core::mem::MaybeUninit;

    use super::{TryRecvError, TrySendError};
    use crate::thread::{Thread, ThreadList, ThreadState};

    pub enum SyncChannelState {
        Idle,
        WithBlockedSenders(ThreadList),
        WithBlockedReceivers(ThreadList),
    }

    pub struct SyncChannel<T>
    where
        T: Copy,
    {
        state: SyncChannelState,
        _phantom: core::marker::PhantomData<T>,
    }

    impl<'a, T> SyncChannel<T>
    where
        T: Copy,
    {
        pub const fn new() -> SyncChannel<T> {
            SyncChannel {
                state: SyncChannelState::Idle,
                _phantom: core::marker::PhantomData,
            }
        }

        fn try_recv_impl(&mut self) -> Result<T, TryRecvError> {
            match &mut self.state {
                SyncChannelState::WithBlockedSenders(senders) => {
                    #[cfg(test)]
                    println!("recv with senders()");

                    // unwrap always succeeds
                    let sender = senders.lpop().unwrap();
                    #[cfg(test)]
                    println!("recv direct copy()");
                    if let ThreadState::ChannelTxBlocked(ptr) = sender.state {
                        let res = unsafe { *(ptr as *mut T) };
                        sender.set_state(ThreadState::Running);
                        if senders.is_empty() {
                            self.state = SyncChannelState::Idle;
                        }
                        Thread::yield_higher();
                        Ok(res)
                    } else {
                        unreachable!();
                    }
                }
                _ => Err(TryRecvError::WouldBlock),
            }
        }

        pub fn try_recv(&mut self) -> Result<T, TryRecvError> {
            critical_section::with(|_| self.try_recv_impl())
        }

        pub fn recv(&mut self) -> T {
            critical_section::with(|_| {
                if let Ok(res) = self.try_recv_impl() {
                    res
                } else {
                    if let SyncChannelState::Idle = self.state {
                        self.state = SyncChannelState::WithBlockedReceivers(ThreadList::new());
                    }
                    let mut res = MaybeUninit::uninit();
                    if let SyncChannelState::WithBlockedReceivers(waiters) = &mut self.state {
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
            if let SyncChannelState::WithBlockedReceivers(waiters) = &mut self.state {
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
                        self.state = SyncChannelState::Idle;
                    }
                } else {
                    unreachable!();
                }
                Ok(())
            } else {
                #[cfg(test)]
                println!("try_send() wouldblock");
                Err(TrySendError::WouldBlock)
            }
        }

        pub fn try_send(&mut self, msg: T) -> Result<(), TrySendError> {
            critical_section::with(|_| self.try_send_impl(msg))
        }

        pub fn send(&mut self, msg: T) {
            critical_section::with(|_| {
                if self.try_send_impl(msg).is_err() {
                    match self.state {
                        SyncChannelState::Idle => (),
                        _ => self.state = SyncChannelState::WithBlockedSenders(ThreadList::new()),
                    }
                    if let SyncChannelState::WithBlockedSenders(waiters) = &mut self.state {
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
            0
        }

        pub fn available(&self) -> usize {
            match self.state {
                // FIXME: count list?
                SyncChannelState::WithBlockedSenders(_) => 1,
                _ => 0,
            }
        }
    }

    impl<T> super::Channel<T> for SyncChannel<T>
    where
        T: Copy,
    {
        fn send(&mut self, msg: T) {
            SyncChannel::send(self, msg)
        }
        fn try_send(&mut self, msg: T) -> Result<(), TrySendError> {
            SyncChannel::try_send(self, msg)
        }
        fn recv(&mut self) -> T {
            SyncChannel::recv(self)
        }
        fn try_recv(&mut self) -> Result<T, TryRecvError> {
            SyncChannel::try_recv(self)
        }
    }
}

/// C bindings for BufferedChannel to support RIOT's "mbox" API
///
/// ("msg" is implemented in thread.rs)
pub mod c {
    #![allow(non_camel_case_types)]
    use super::BufferedChannel;
    use crate::thread::c::msg_t;
    use core::mem::MaybeUninit;
    use ref_cast::RefCast;

    #[derive(RefCast)]
    #[repr(transparent)]
    pub struct mbox_t(BufferedChannel<'static, msg_t>);

    pub const MBOX_T_SIZEOF: usize = 24;
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
            0: BufferedChannel::new(Some(queue)),
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
    use core::mem::MaybeUninit;

    static mut STACK: [u8; 1024] = [0; 1024];

    fn func(arg: usize) {
        let channel = unsafe { &mut *(arg as *mut BufferedChannel<u32>) };
        for i in 0..4u32 {
            channel.send(i);
        }
    }

    let mut array: [MaybeUninit<u32>; 4] = unsafe { MaybeUninit::uninit().assume_init() };
    let mut channel = BufferedChannel::new(Some(&mut array));

    assert_eq!(channel.try_recv(), Err(TryRecvError::WouldBlock));

    unsafe {
        Thread::create(
            &mut STACK,
            func,
            &channel as *const BufferedChannel<u32> as usize,
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
    use core::mem::MaybeUninit;
    use riot_rs_rt::debug::println;

    static mut STACK: [u8; 1024] = [0; 1024];

    fn func(arg: usize) {
        let channel = unsafe { &mut *(arg as *mut BufferedChannel<u32>) };
        for i in 0..8u32 {
            channel.send(i);
        }
    }

    let mut array: [MaybeUninit<u32>; 4] = unsafe { MaybeUninit::uninit().assume_init() };
    let mut channel = BufferedChannel::new(Some(&mut array));

    assert_eq!(channel.try_recv(), Err(TryRecvError::WouldBlock));

    unsafe {
        Thread::create(
            &mut STACK,
            func,
            &channel as *const BufferedChannel<u32> as usize,
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

#[test_case]
fn test_sync_channel_lowprio_sender() {
    use crate::thread::{CreateFlags, Thread};

    static mut STACK: [u8; 1024] = [0; 1024];

    fn func(arg: usize) {
        let channel = unsafe { &mut *(arg as *mut SyncChannel<u32>) };
        for i in 0..4u32 {
            channel.send(i);
        }
    }

    let mut channel = SyncChannel::new();

    assert_eq!(channel.try_recv(), Err(TryRecvError::WouldBlock));

    unsafe {
        Thread::create(
            &mut STACK,
            func,
            &channel as *const SyncChannel<u32> as usize,
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
