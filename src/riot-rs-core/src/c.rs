/// C bindings and glue code
pub mod c {
    use core::ffi::{c_char, c_void};
    use core::unimplemented;
    use critical_section::{self};

    pub use embedded_threads::{RunqueueId, Thread, ThreadFlags, ThreadId, ThreadState};
    pub use ref_cast::RefCast;

    #[derive(RefCast)]
    #[repr(transparent)]
    pub struct thread_t(Thread);

    pub use crate::Lock;

    #[no_mangle]
    pub static mut sched_context_switch_request: bool = false;

    // pub const THREAD_CREATE_SLEEPING: u32 = CreateFlags::SLEEPING.bits;
    // pub const THREAD_CREATE_WOUT_YIELD: u32 = CreateFlags::WITHOUT_YIELD.bits;
    // pub const THREAD_CREATE_STACKTEST: u32 = CreateFlags::STACKTEST.bits;
    pub const THREAD_CREATE_SLEEPING: u32 = 1 << 0;
    pub const THREAD_CREATE_WOUT_YIELD: u32 = 1 << 1;
    pub const THREAD_CREATE_STACKTEST: u32 = 1 << 2;

    // cbindgen cannot export these
    //pub const SCHED_PRIO_LEVELS: u32 = embedded_threads::SCHED_PRIO_LEVELS;
    //pub const THREADS_NUMOF: u32 = embedded_threads::THREADS_NUMOF;
    pub const SCHED_PRIO_LEVELS: u32 = 8;
    pub const THREADS_NUMOF: u32 = 8;

    #[no_mangle]
    pub unsafe extern "C" fn _thread_create(
        stack_ptr: &'static mut c_char,
        stack_size: usize,
        priority: u8,
        flags: u32,
        thread_func: usize,
        arg: usize,
        _name: &'static c_char,
    ) -> ThreadId {
        // let stack_ptr = stack_ptr as *mut c_char as usize as *mut u8;
        // // println!(
        // //     "stack_ptr as u8: {:#x} size: {}",
        // //     stack_ptr as usize, stack_size
        // // );

        // // align end of stack (lowest address)
        // let misalign = stack_ptr as usize & 0x7;
        // let mut stack_ptr = stack_ptr;
        // let mut stack_size = stack_size;
        // if misalign > 0 {
        //     stack_ptr = (stack_ptr as usize + 8 - misalign) as *mut u8;
        //     stack_size -= 8 - misalign;
        // }

        // // align start of stack (lowest address plus stack_size)
        // stack_size &= !0x7;

        // let stack = core::slice::from_raw_parts_mut(stack_ptr, stack_size);

        unimplemented!();
        0
    }

    #[no_mangle]
    pub extern "C" fn thread_get_active() -> &'static mut thread_t {
        unimplemented!();
        // thread_t::ref_cast_mut(Thread::current())
    }

    #[no_mangle]
    pub unsafe extern "C" fn thread_get(pid: ThreadId) -> *mut thread_t {
        // if Thread::pid_is_valid(pid) {
        //     thread_t::ref_cast_mut(Thread::get_mut(pid)) as *mut thread_t
        // } else {
        //     core::ptr::null_mut()
        // }
        unimplemented!();
        core::ptr::null_mut()
    }

    #[no_mangle]
    pub unsafe extern "C" fn thread_wakeup(pid: ThreadId) {
        unimplemented!();
        // Thread::wakeup(pid)
    }

    #[no_mangle]
    pub extern "C" fn thread_yield_higher() {
        unimplemented!();
        // Thread::yield_higher();
    }

    #[no_mangle]
    pub extern "C" fn thread_yield() {
        unimplemented!();
        // Thread::yield_next();
    }

    #[no_mangle]
    pub extern "C" fn thread_getpid() -> ThreadId {
        unimplemented!();
        // Thread::current_pid()
        0
    }

    #[no_mangle]
    pub unsafe extern "C" fn _core_panic(_panic_code: usize, _msg: &'static c_char) -> ! {
        panic!("rust core_panic()");
    }

    // #[no_mangle]
    // pub unsafe extern "C" fn MUTEX_INIT() -> mutex_t {
    //     let lock = Lock::new();
    //     core::mem::transmute(lock)
    // }

    // cbindgen cannot export these
    //pub const MUTEX_T_SIZEOF: usize = core::mem::sizeof::<Lock>();
    //pub const MUTEX_T_ALIGNOF: usize = core::mem::align_of::<Lock>();
    pub const MUTEX_T_SIZEOF: usize = 8;
    pub const MUTEX_T_ALIGNOF: usize = 4;

    #[no_mangle]
    pub static MUTEX_INIT: Lock = Lock::new();

    #[no_mangle]
    pub static MUTEX_INIT_LOCKED: Lock = Lock::new_locked();

    #[no_mangle]
    pub extern "C" fn mutex_init(mutex: &mut Lock) {
        *mutex = Lock::new()
    }

    #[no_mangle]
    pub extern "C" fn mutex_init_locked(mutex: &mut Lock) {
        *mutex = Lock::new_locked()
    }

    #[no_mangle]
    pub extern "C" fn mutex_lock(mutex: &mut Lock) {
        mutex.acquire()
    }

    #[no_mangle]
    pub extern "C" fn mutex_trylock(mutex: &mut Lock) -> bool {
        mutex.try_acquire()
    }

    #[no_mangle]
    pub extern "C" fn mutex_unlock(mutex: &mut Lock) {
        mutex.release()
    }

    #[no_mangle]
    pub extern "C" fn mutex_unlock_and_sleep(mutex: &mut Lock) {
        critical_section::with(|_| {
            mutex.release();
            Thread::sleep();
        });
    }

    #[repr(C)]
    pub struct mutex_cancel_t {
        lock: &'static Lock,
        pid: ThreadId,
        //        cancelled: AtomicBool,
    }

    #[no_mangle]
    pub extern "C" fn mutex_cancel_init(mutex: &'static Lock) -> mutex_cancel_t {
        unimplemented!();
    }

    #[no_mangle]
    pub extern "C" fn mutex_cancel(mutex_cancel: &mutex_cancel_t) {
        unimplemented!();
    }

    #[no_mangle]
    pub extern "C" fn mutex_lock_cancelable(mutex_cancel: &mutex_cancel_t) -> i32 {
        unimplemented!();
    }

    // we need to put both a ptr and a value in here.
    // but ffi::c_void is not copy. This needs to be Copy to be used
    // with mbox. so make "ptr" a usize field, and manually add
    // msg_content_t to msg.h (tell cbindgen to ignore this)
    /// cbindgen:ignore
    #[repr(C)]
    #[derive(Copy, Clone)]
    pub union msg_content_t {
        pub value: u32,
        pub ptr: usize,
    }

    impl core::fmt::Debug for msg_content_t {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.debug_struct("msg_content_t(not shown)").finish()
        }
    }

    /// cbindgen:field-names=[sender_pid, type, content]
    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    pub struct msg_t {
        pub sender_pid: ThreadId,
        pub type_: u16,
        pub content: msg_content_t,
    }

    impl core::default::Default for msg_t {
        fn default() -> Self {
            msg_t {
                sender_pid: ThreadId::default(),
                type_: 0u16,
                content: msg_content_t { value: 0 },
            }
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn msg_send(msg: &mut msg_t, target_pid: ThreadId) -> i32 {
        unimplemented!();
        0
    }

    #[no_mangle]
    pub unsafe extern "C" fn msg_receive(msg: &mut msg_t) {
        unimplemented!();
    }

    #[no_mangle]
    pub unsafe extern "C" fn msg_try_receive(msg: &mut msg_t) -> bool {
        unimplemented!();
    }

    #[no_mangle]
    pub unsafe extern "C" fn msg_send_receive(
        msg: *mut msg_t,
        reply: *mut msg_t,
        target_pid: ThreadId,
    ) -> bool {
        unimplemented!();
    }

    #[no_mangle]
    pub unsafe extern "C" fn msg_reply(msg: &mut msg_t, reply: &mut msg_t) -> i32 {
        unimplemented!();
    }

    #[no_mangle]
    pub unsafe extern "C" fn msg_try_send(msg: &mut msg_t, target_pid: ThreadId) -> bool {
        unimplemented!();
    }

    #[no_mangle]
    pub unsafe extern "C" fn msg_init_queue(array: &'static mut msg_t, num: usize) {
        unimplemented!();
    }

    #[no_mangle]
    pub unsafe extern "C" fn msg_send_to_self(msg: &mut msg_t) -> i32 {
        unimplemented!();
    }

    #[no_mangle]
    pub unsafe extern "C" fn thread_has_msg_queue(thread: &Thread) -> bool {
        unimplemented!();
    }

    #[no_mangle]
    pub unsafe extern "C" fn msg_avail() -> i32 {
        unimplemented!();
    }

    #[no_mangle]
    pub unsafe extern "C" fn thread_flags_set(thread: &mut Thread, mask: ThreadFlags) {
        unimplemented!();
    }

    #[no_mangle]
    pub unsafe extern "C" fn thread_flags_wait_any(mask: ThreadFlags) -> ThreadFlags {
        unimplemented!();
    }

    #[no_mangle]
    pub unsafe extern "C" fn thread_flags_wait_one(mask: ThreadFlags) -> ThreadFlags {
        unimplemented!();
    }

    #[no_mangle]
    pub unsafe extern "C" fn thread_flags_wait_all(mask: ThreadFlags) -> ThreadFlags {
        unimplemented!();
    }

    #[no_mangle]
    pub unsafe extern "C" fn thread_flags_clear(mask: ThreadFlags) -> ThreadFlags {
        unimplemented!();
    }

    #[no_mangle]
    pub unsafe extern "C" fn thread_zombify() {
        unimplemented!();
    }

    #[no_mangle]
    pub unsafe extern "C" fn thread_kill_zombie(pid: ThreadId) -> i32 {
        unimplemented!();
    }

    #[derive(Debug)]
    #[repr(C)]
    pub enum thread_status_t {
        Invalid,
        Running,
        Paused,
        Zombie,
        MutexBlocked,
        FlagBlockedAny,
        FlagBlockedAll,
        ChannelRxBlocked,
        ChannelTxBlocked,
        ChannelReplyBlocked,
        ChannelTxReplyBlocked,
    }

    impl core::convert::From<ThreadState> for thread_status_t {
        fn from(item: ThreadState) -> thread_status_t {
            match item {
                ThreadState::Invalid => thread_status_t::Invalid,
                ThreadState::Running => thread_status_t::Running,
                ThreadState::Paused => thread_status_t::Paused,
                ThreadState::Zombie => thread_status_t::Zombie,
            }
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn thread_state_to_string(state: thread_status_t) -> *const c_char {
        unimplemented!();
        // let res = match state {
        //     thread_status_t::Running => b"pending\0".as_ptr(),
        //     thread_status_t::Zombie => b"zombie\0".as_ptr(),
        //     thread_status_t::Paused => b"sleeping\0".as_ptr(),
        //     thread_status_t::MutexBlocked => b"bl mutex\0".as_ptr(),
        //     thread_status_t::FlagBlockedAny => b"bl anyfl\0".as_ptr(),
        //     thread_status_t::FlagBlockedAll => b"bl allfl\0".as_ptr(),
        //     thread_status_t::ChannelTxBlocked => b"bl send\0".as_ptr(),
        //     thread_status_t::ChannelRxBlocked => b"bl rx\0".as_ptr(),
        //     thread_status_t::ChannelTxReplyBlocked => b"bl txrx\0".as_ptr(),
        //     thread_status_t::ChannelReplyBlocked => b"bl reply\0".as_ptr(),
        //     _ => b"unknown\0".as_ptr(),
        // };
        // res as *const u8 as usize as *const c_char
    }

    #[no_mangle]
    pub unsafe extern "C" fn thread_get_status(thread: &Thread) -> thread_status_t {
        thread_status_t::from(thread.state)
    }

    #[no_mangle]
    pub unsafe extern "C" fn thread_getpid_of(thread: &Thread) -> ThreadId {
        thread.pid
    }

    #[no_mangle]
    pub unsafe extern "C" fn thread_get_priority(thread: &Thread) -> RunqueueId {
        thread.prio
    }

    #[no_mangle]
    pub unsafe extern "C" fn thread_is_active(thread: &Thread) -> bool {
        thread.state == ThreadState::Running
    }

    #[no_mangle]
    pub unsafe extern "C" fn thread_get_sp(thread: &Thread) -> *const c_void {
        thread.sp as *const c_void
    }

    #[no_mangle]
    pub unsafe extern "C" fn thread_get_stackstart(thread: &Thread) -> *mut c_void {
        thread.stack_bottom() as *mut c_void
    }

    #[no_mangle]
    pub unsafe extern "C" fn thread_get_stacksize(thread: &Thread) -> usize {
        thread.stack_size()
    }

    #[no_mangle]
    pub unsafe extern "C" fn thread_get_name(thread: &Thread) -> *const c_char {
        unimplemented!();
    }

    #[no_mangle]
    pub unsafe extern "C" fn thread_measure_stack_free(start: *const c_void) -> usize {
        // assume proper alignment
        assert!((start as usize & 0x3) == 0);
        let mut pos = start as usize;
        while *(pos as *const usize) == pos as usize {
            pos = pos + core::mem::size_of::<usize>();
        }
        pos as usize - start as usize
    }

    #[no_mangle]
    pub unsafe extern "C" fn thread_isr_stack_start() -> *mut c_void {
        0 as *mut c_void
    }

    #[no_mangle]
    pub unsafe extern "C" fn thread_isr_stack_pointer() -> *mut c_void {
        0 as *mut c_void
    }

    #[no_mangle]
    pub unsafe extern "C" fn thread_isr_stack_size() -> usize {
        0
    }

    #[no_mangle]
    pub unsafe extern "C" fn thread_isr_stack_usage() -> usize {
        0
    }

    #[no_mangle]
    pub unsafe extern "C" fn cpu_switch_context_exit() {
        embedded_threads::start_threading();
    }
}
