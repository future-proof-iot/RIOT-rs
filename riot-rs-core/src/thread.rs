use core::ptr::write_volatile;
use core::sync::atomic::{AtomicUsize, Ordering};

use cortex_m::interrupt;
use cortex_m::peripheral::SCB;

use bitflags::bitflags;

use clist::Link;

use cfg_if::cfg_if;
cfg_if! {
    if #[cfg(feature = "riot-rs-rt")] {
        pub(crate) use riot_rs_rt::debug::{println};
    } else {
        //use cortex_m_semihosting::hprint as print;
        pub(crate) use cortex_m_semihosting::hprintln as println;
    }
}

pub const SCHED_PRIO_LEVELS: usize = 16;

pub const THREADS_NUMOF: usize = 16;
pub const THREAD_FLAG_TIMEOUT: ThreadFlags = (1 as ThreadFlags) << 14;

pub type Pid = u16;
pub type ThreadFlags = u16;

#[derive(Copy, Clone)]
pub struct Thread {
    sp: usize,
    high_regs: [usize; 8],
    list_entry: Link,
    pub(crate) state: ThreadState,
    pub prio: u8,
    pub flags: ThreadFlags,
    pub pid: Pid,
}

#[derive(Copy, Clone, PartialEq)]
pub enum ThreadState {
    Invalid,
    Running,
    Paused,
    MutexBlocked,
    MsgBlocked,
    FlagBlocked(FlagWaitMode),
    ChannelRxBlocked(usize),
    ChannelTxBlocked(usize),
}

pub(crate) type ThreadList = clist::TypedList<Thread, { clist::offset_of!(Thread, list_entry) }>;

bitflags! {
    #[derive(Default)]
    #[repr(C)]
    pub struct CreateFlags: u32 {
        const SLEEPING      = 0b00000001;
        const WITHOUT_YIELD = 0b00000010;
        const STACKTEST     = 0b00000100;
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Msg {
    pub a: u32,
    pub b: u32,
    pub c: u32,
    pub d: u32,
}

use crate::runqueue::RunQueue;

static mut RUNQUEUE: RunQueue<SCHED_PRIO_LEVELS> = RunQueue::new();

#[no_mangle]
unsafe fn sched(old_sp: usize) {
    let mut current = Thread::current();

    let next_pid = RUNQUEUE.get_next() as Pid;

    let next = Thread::get(next_pid);

    if next as *const Thread == current as *const Thread {
        llvm_asm!("" :: "{r0}"(0)::"volatile");
        return;
    }

    current.sp = old_sp;
    CURRENT_THREAD.store((next as *const Thread) as usize, Ordering::Release);

    // PendSV expects these three pointers in r1, r2 and r3
    // write to registers manually, as ABI would return the values via stack
    llvm_asm!("" :: "{r0}"(current.high_regs.as_ptr()), "{r1}"(next.high_regs.as_ptr()), "{r2}"(next.sp) :: "volatile" );
}

static mut THREADS: [Thread; THREADS_NUMOF] = [Thread {
    sp: 0,
    state: ThreadState::Invalid,
    list_entry: Link::new(),
    high_regs: [0; 8],
    prio: 0,
    pid: 0,
    flags: 0,
}; THREADS_NUMOF];

static CURRENT_THREAD: AtomicUsize = AtomicUsize::new(0);

pub fn cleanup() -> ! {
    let current = Thread::current();
    //hprintln!("thread {} ended.", current.pid);
    current.set_state(ThreadState::Invalid);
    Thread::yield_next();

    loop {}
}

/// Supervisor Call exception handler
///
/// This is currently used to initiate threading.
#[naked]
#[no_mangle]
#[allow(non_snake_case)]
unsafe fn SVCall() {
    llvm_asm!(
            "
            movw LR, #0xFFFd
            movt LR, #0xFFFF
            "
            :::: "volatile" );
}

/// PendSV exception handler
#[naked]
#[no_mangle]
#[allow(non_snake_case)]
unsafe fn PendSV() {
    llvm_asm!(
            "
            mrs r0, psp
            bl sched
            cmp r0, #0
            beq return
            stmia r0, {r4-r11}
            ldmia r1, {r4-r11}
            msr.n psp, r2
            return:
            movw LR, #0xFFFd
            movt LR, #0xFFFF
            "
            :::: "volatile" );
}

impl Thread {
    /// Create a new thread
    pub fn create(
        stack: &mut [u8],
        func: fn(arg: usize),
        arg: usize,
        prio: u8,
        flags: CreateFlags,
    ) -> &Thread {
        Thread::create_(stack, func as usize, arg, prio, flags)
    }

    ///
    fn create_(stack: &mut [u8], func: usize, arg: usize, prio: u8, flags: CreateFlags) -> &Thread {
        unsafe {
            let unused_pid = Thread::find_unused_pid().unwrap();
            let mut thread = &mut THREADS[unused_pid as usize];
            thread.sp = Thread::setup_stack(stack, func, arg);
            thread.pid = unused_pid;
            thread.prio = prio;

            if flags.contains(CreateFlags::SLEEPING) {
                thread.state = ThreadState::Paused;
            } else {
                thread.state = ThreadState::Running;
                RUNQUEUE.add(unused_pid as usize, thread.prio as usize);
                if !flags.contains(CreateFlags::WITHOUT_YIELD) {
                    Thread::yield_higher();
                }
            }

            return thread;
        }
    }

    pub unsafe fn get(pid: Pid) -> &'static Thread {
        return &THREADS[pid as usize];
    }

    pub unsafe fn get_mut(pid: Pid) -> &'static mut Thread {
        return &mut THREADS[pid as usize];
    }

    pub fn current() -> &'static mut Thread {
        unsafe {
            return &mut *(CURRENT_THREAD.load(Ordering::Acquire) as *mut Thread);
        }
    }

    pub fn current_pid() -> Pid {
        Thread::current().pid
    }

    pub fn set_state(&mut self, state: ThreadState) {
        let old_state = self.state;
        self.state = state;
        if old_state != ThreadState::Running && state == ThreadState::Running {
            unsafe {
                RUNQUEUE.add(self.pid as usize, self.prio as usize);
            }
        } else if old_state == ThreadState::Running && state != ThreadState::Running {
            unsafe {
                RUNQUEUE.del(self.pid as usize, self.prio as usize);
            }
        }
    }

    pub unsafe fn jump_to(&self) {
        CURRENT_THREAD.store((self as *const Thread) as usize, Ordering::Release);
        llvm_asm!(
            "
            msr psp, r1
            svc 0
            "
        :
        : "{r1}"(self.sp)
        :
        : "volatile" );
    }

    //#[inline]
    pub fn yield_next() {
        let current = Thread::current();
        unsafe {
            RUNQUEUE.advance(current.pid as u8, current.prio as usize);
            SCB::set_pendsv();
            cortex_m::asm::isb();
        }
    }

    pub fn yield_higher() {
        SCB::set_pendsv();
        cortex_m::asm::isb();
    }

    pub fn isr_enable_disable() {
        core::sync::atomic::compiler_fence(Ordering::SeqCst);
        unsafe { cortex_m::interrupt::enable() };
        cortex_m::asm::isb();
        cortex_m::interrupt::disable();
    }

    pub fn sleep() {
        Thread::current().set_state(ThreadState::Paused);
        Thread::yield_higher();
    }

    pub fn wakeup(pid: Pid) {
        unsafe {
            Thread::get_mut(pid)._wakeup();
        }
    }

    pub(crate) fn _wakeup(&mut self) {
        assert!(self.state == ThreadState::Paused);
        self.set_state(ThreadState::Running);
        Thread::yield_higher();
    }

    /// Sets up stack for newly created threads.
    ///
    /// After running this, the stack should look as if the thread was
    /// interrupted by an ISR. On the next return, it starts executing
    /// `func`.
    fn setup_stack(stack: &mut [u8], func: usize, arg: usize) -> usize {
        let stack_start = stack.as_ptr() as usize;
        let stack_pos = (stack_start + stack.len() - 36) as *mut usize;

        unsafe {
            write_volatile(stack_pos.offset(0), arg); // -> R0
            write_volatile(stack_pos.offset(1), 1); // -> R1
            write_volatile(stack_pos.offset(2), 2); // -> R2
            write_volatile(stack_pos.offset(3), 3); // -> R3
            write_volatile(stack_pos.offset(4), 12); // -> R12
            write_volatile(stack_pos.offset(5), cleanup as usize); // -> LR
            write_volatile(stack_pos.offset(6), func); // -> PC
            write_volatile(stack_pos.offset(7), 0x01000000); // -> APSR
        }

        return stack_pos as usize;
    }

    /// Find an unused PID
    unsafe fn find_unused_pid() -> Option<Pid> {
        for i in 0..THREADS_NUMOF {
            if THREADS[i].state == ThreadState::Invalid {
                return Some(i as Pid);
            }
        }
        None
    }

    /// receive a message, blocking
    pub fn receive_msg(&mut self) -> Msg {
        interrupt::free(|_| {
            let r0: u32;
            let r1: u32;
            let r2: u32;
            let r3: u32;

            self.set_state(ThreadState::MsgBlocked);
            Thread::yield_higher();

            unsafe {
                llvm_asm!(
            "
            "
            : "={r0}"(r0), "={r1}"(r1), "={r2}"(r2), "={r3}"(r3)
            :
            :
            : "volatile" );
            };

            Msg {
                a: r0,
                b: r1,
                c: r2,
                d: r3,
            }
        })
    }

    pub fn send_msg(m: Msg, target: &mut Thread) -> bool {
        impl Thread {
            unsafe fn write_regs(&mut self, r0: u32, r1: u32, r2: u32, r3: u32) {
                let sp = self.sp as *mut u32;
                write_volatile(sp.offset(0), r0); // -> R0
                write_volatile(sp.offset(1), r1); // -> R1
                write_volatile(sp.offset(2), r2); // -> R2
                write_volatile(sp.offset(3), r3); // -> R3
            }
        }
        interrupt::free(|_| {
            if target.state == ThreadState::MsgBlocked {
                // this is fine, correct thread state checked above
                unsafe { target.write_regs(m.a, m.b, m.c, m.d) };
                target.set_state(ThreadState::Running);
                Thread::yield_higher();
                return true;
            }
            false
        })
    }

    /// Put thread in waitlist with given state
    pub(crate) fn wait_on(&mut self, list: &mut ThreadList, wait_state: ThreadState) {
        list.rpush(self);
        self.set_state(wait_state);
        Thread::yield_higher();
    }

    /// Start riot-rs-core scheduler
    ///
    /// Note: this _must only be called once during startup_.
    pub unsafe fn start_threading() -> ! {
        let next_pid = RUNQUEUE.get_next() as Pid;
        Thread::get(next_pid).jump_to();
        loop {}
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum FlagWaitMode {
    Any(ThreadFlags),
    All(ThreadFlags),
}

impl Thread {
    pub fn flag_set(&mut self, mask: ThreadFlags) {
        interrupt::free(|_| {
            self.flags |= mask;
            if match self.state {
                ThreadState::FlagBlocked(mode) => match mode {
                    FlagWaitMode::Any(bits) => (self.flags & bits != 0),
                    FlagWaitMode::All(bits) => (self.flags & bits == bits),
                },
                _ => false,
            } {
                self.set_state(ThreadState::Running);
                Thread::yield_higher();
            }
        })
    }

    pub fn flag_wait_all(mask: ThreadFlags) -> ThreadFlags {
        let thread = Thread::current();
        loop {
            if let Some(result) = cortex_m::interrupt::free(|_| {
                if thread.flags & mask != 0 {
                    let result = thread.flags & mask;
                    thread.flags &= !mask;
                    Some(result)
                } else {
                    None
                }
            }) {
                return result;
            } else {
                thread.set_state(ThreadState::FlagBlocked(FlagWaitMode::All(mask)));
                Thread::yield_higher();
            }
        }
    }

    pub fn flag_wait_any(mask: ThreadFlags) -> ThreadFlags {
        let thread = Thread::current();
        loop {
            if let Some(result) = cortex_m::interrupt::free(|_| {
                if thread.flags & mask != 0 {
                    let res = thread.flags & mask;
                    thread.flags &= !res;
                    Some(res)
                } else {
                    None
                }
            }) {
                return result;
            } else {
                thread.set_state(ThreadState::FlagBlocked(FlagWaitMode::Any(mask)));
                Thread::yield_higher();
            }
        }
    }

    pub fn flag_wait_one(mask: ThreadFlags) -> ThreadFlags {
        let thread = Thread::current();
        loop {
            if let Some(result) = cortex_m::interrupt::free(|_| {
                if thread.flags & mask != 0 {
                    let mut res = thread.flags & mask;
                    // clear all but least significant bit
                    res &= !res + 1;
                    thread.flags &= !res;
                    Some(res)
                } else {
                    None
                }
            }) {
                return result;
            } else {
                thread.set_state(ThreadState::FlagBlocked(FlagWaitMode::Any(mask)));
                Thread::yield_higher();
            }
        }
    }

    pub fn flag_clear(mask: ThreadFlags) -> ThreadFlags {
        let thread = Thread::current();
        cortex_m::interrupt::free(|_| {
            let res = thread.flags & mask;
            thread.flags &= !mask;
            res
        })
    }
}
// this block contains experimental closure thread support
impl Thread {
    pub fn setup_stack_noregs(stack: &mut [u8], func: usize) -> usize {
        let stack_start = stack.as_ptr() as usize;
        let stack_pos = (stack_start + stack.len() - 36) as *mut usize;

        unsafe {
            write_volatile(stack_pos.offset(4), 12); // -> R12
            write_volatile(stack_pos.offset(5), cleanup as usize); // -> LR
            write_volatile(stack_pos.offset(6), func); // -> PC
            write_volatile(stack_pos.offset(7), 0x01000000); // -> APSR
        }

        return stack_pos as usize;
    }

    // create with passing data on stack. used by spawn()
    fn _create(stack: &mut [u8], func: usize, prio: u8, data: &[u8]) -> &'static Thread {
        unsafe {
            let unused_pid = Thread::find_unused_pid().unwrap();
            let mut thread = &mut THREADS[unused_pid as usize];
            if data.len() > 16 {
                let (stack, data_in_stack) = stack.split_at_mut(stack.len() - data.len());
                assert_eq!(data.len(), data_in_stack.len());
                data_in_stack.copy_from_slice(data);
                thread.sp = Thread::setup_stack(stack, func, data.as_ptr() as usize);
            } else {
                let stack_len = stack.len();
                let data_in_stack = &mut stack[stack_len - 36..stack_len - (36 - data.len())];
                data_in_stack.copy_from_slice(data);
                thread.sp = Thread::setup_stack_noregs(stack, func);
            }
            thread.pid = unused_pid;
            thread.prio = prio;

            thread.state = ThreadState::Running;
            RUNQUEUE.add(unused_pid as usize, thread.prio as usize);

            return thread;
        }
    }

    /// spawn thread using closure.
    pub fn spawn<F, T>(stack: &'static mut [u8], f: F) -> &Thread
    where
        F: FnOnce() -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        let f_data = unsafe {
            core::slice::from_raw_parts(&f as *const F as *const u8, core::mem::size_of::<F>())
        };

        Thread::_create(stack, F::call_once as usize, 6, f_data)
    }
}

pub mod c {
    #![allow(non_camel_case_types)]
    use ref_cast::RefCast;

    use super::println;
    use crate::lock::Lock;
    use crate::thread::{CreateFlags, Msg, Pid, Thread, ThreadFlags};

    #[derive(RefCast)]
    #[repr(transparent)]
    pub struct thread_t(Thread);
    pub struct c_char(u8);

    #[no_mangle]
    pub static mut sched_context_switch_request: bool = false;

    // pub const THREAD_CREATE_SLEEPING: u32 = CreateFlags::SLEEPING.bits;
    // pub const THREAD_CREATE_WOUT_YIELD: u32 = CreateFlags::WITHOUT_YIELD.bits;
    // pub const THREAD_CREATE_STACKTEST: u32 = CreateFlags::STACKTEST.bits;
    pub const THREAD_CREATE_SLEEPING: u32 = 1 << 0;
    pub const THREAD_CREATE_WOUT_YIELD: u32 = 1 << 1;
    pub const THREAD_CREATE_STACKTEST: u32 = 1 << 2;

    #[no_mangle]
    pub unsafe extern "C" fn _thread_create(
        stack_ptr: &'static mut c_char,
        stack_size: usize,
        priority: u8,
        flags: u32,
        thread_func: usize,
        arg: usize,
        _name: &'static c_char,
    ) -> Pid {
        let stack_ptr = stack_ptr as *mut c_char as usize as *mut u8;
        //println!("stack_ptr as u8: {:#x}", stack_ptr as usize);
        let stack = core::slice::from_raw_parts_mut(stack_ptr, stack_size);
        let thread = Thread::create_(
            stack,
            thread_func,
            arg,
            priority,
            CreateFlags::from_bits_truncate(flags),
        );
        thread.pid
    }

    #[no_mangle]
    pub extern "C" fn thread_get_active() -> &'static mut thread_t {
        thread_t::ref_cast_mut(Thread::current())
    }

    #[no_mangle]
    pub unsafe extern "C" fn thread_get(pid: Pid) -> &'static mut thread_t {
        thread_t::ref_cast_mut(Thread::get_mut(pid))
    }

    #[no_mangle]
    pub unsafe extern "C" fn thread_wakeup(pid: Pid) {
        Thread::wakeup(pid)
    }

    #[no_mangle]
    pub extern "C" fn thread_get_stack_size(_thread: &'static mut thread_t) -> usize {
        0usize
    }

    #[no_mangle]
    pub extern "C" fn thread_yield_higher() {
        Thread::yield_higher();
    }

    #[no_mangle]
    pub extern "C" fn thread_yield() {
        Thread::yield_next();
    }

    #[no_mangle]
    pub extern "C" fn thread_getpid() -> Pid {
        Thread::current_pid()
    }

    #[no_mangle]
    pub unsafe extern "C" fn cpu_switch_context_exit() -> ! {
        llvm_asm!( "cpsie   i\n" :::: "volatile" );
        Thread::start_threading()
    }

    #[no_mangle]
    pub unsafe extern "C" fn _core_panic(_panic_code: usize, _msg: &'static c_char) -> ! {
        panic!("rust core_panic()");
    }

    #[no_mangle]
    pub extern "C" fn rust_hello() {
        println!("Hello from Rust!");
    }

    // #[no_mangle]
    // pub unsafe extern "C" fn MUTEX_INIT() -> mutex_t {
    //     let lock = Lock::new();
    //     core::mem::transmute(lock)
    // }

    #[no_mangle]
    pub extern "C" fn mutex_init(mutex: &mut Lock) {
        *mutex = Lock::new()
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
        cortex_m::interrupt::free(|_| {
            mutex.release();
            Thread::sleep();
        });
    }

    // we need to put both a ptr and a value in here.
    // but ffi::c_void is not copy. This needs to be Copy to be used
    // with mbox. so make "ptr" a usize field, and manually add
    // msg_content_t to msg.h (tell cbindgen to ignore this)
    /// cbindgen:ignore
    #[repr(C)]
    #[derive(Copy, Clone)]
    pub union msg_content_t {
        value: u32,
        ptr: usize,
    }

    /// cbindgen:field-names=[sender_pid, type, content]
    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct msg_t {
        sender_pid: Pid,
        _type: u16,
        content: msg_content_t,
    }

    #[no_mangle]
    pub unsafe extern "C" fn msg_send(msg: &'static mut msg_t, target_pid: Pid) {
        msg.sender_pid = Thread::current_pid();
        let target = Thread::get_mut(target_pid);
        let msg_ = Msg {
            a: msg.sender_pid as u32,
            b: msg._type as u32,
            c: msg.content.value,
            d: 0,
        };
        Thread::send_msg(msg_, target);
    }

    #[no_mangle]
    pub unsafe extern "C" fn msg_receive(msg: &'static mut msg_t) {
        let msg_ = Thread::current().receive_msg();
        msg.sender_pid = msg_.a as Pid;
        msg._type = msg_.b as u16;
        msg.content = msg_content_t { value: msg_.c };
    }

    #[no_mangle]
    pub unsafe extern "C" fn msg_try_receive(_msg: &'static mut msg_t) -> bool {
        unimplemented!();
    }

    #[no_mangle]
    pub unsafe extern "C" fn thread_flags_set(thread: &mut Thread, mask: ThreadFlags) {
        thread.flag_set(mask)
    }

    #[no_mangle]
    pub unsafe extern "C" fn thread_flags_wait_any(mask: ThreadFlags) -> ThreadFlags {
        Thread::flag_wait_any(mask)
    }

    #[no_mangle]
    pub unsafe extern "C" fn thread_flags_wait_one(mask: ThreadFlags) -> ThreadFlags {
        Thread::flag_wait_one(mask)
    }

    #[no_mangle]
    pub unsafe extern "C" fn thread_flags_wait_all(mask: ThreadFlags) -> ThreadFlags {
        Thread::flag_wait_all(mask)
    }

    #[no_mangle]
    pub unsafe extern "C" fn thread_flags_clear(mask: ThreadFlags) -> ThreadFlags {
        Thread::flag_clear(mask)
    }

    // the C bindings use aligned byte arrays to represent Rust types that can
    // not be expressed in C, but need their size at compile time on the C side.
    // These const values are used to pass the corrext sizes to the C headers.
    // Alas, cbindgen doesn't understand const functions, so we need to hard-code
    // the values.
    //
    // Test cases ensure that the values match.
    pub const MUTEX_T_SIZEOF: usize = 8; //core::mem::size_of::<Lock>();
    pub const MUTEX_T_ALIGNOF: usize = 4; //core::mem::align_of::<Lock>();

    #[test_case]
    fn test_const_defines() {
        assert_eq!(core::mem::size_of::<Lock>(), MUTEX_T_SIZEOF);
        assert_eq!(core::mem::align_of::<Lock>(), MUTEX_T_ALIGNOF);
    }
}

#[test_case]
fn test_pid_is_one() {
    assert!(Thread::current().pid == 1);
}

#[test_case]
fn test_thread_flags() {
    fn func(arg: usize) {
        let thread = arg as *mut Thread;
        let thread = unsafe { &mut *thread };
        thread.flag_set(0b1);
        thread.flag_set(0b1010);
        thread.flag_set(0b100);
        thread.flag_set(0b11111);
    }

    static mut STACK: [u8; 1024] = [0; 1024];

    use crate::thread::{CreateFlags, Thread};

    let thread = Thread::current();

    unsafe {
        Thread::create(
            &mut STACK,
            func,
            thread as *mut Thread as usize,
            1,
            CreateFlags::empty(),
        );
    }

    assert_eq!(thread.flags, 0);
    assert_eq!(Thread::flag_wait_any(0b1), 0b1);
    assert_eq!(thread.flags, 0);
    assert_eq!(Thread::flag_wait_any(0b11), 0b10);
    assert_eq!(thread.flags, 0b1000);
    assert_eq!(Thread::flag_wait_any(0b100), 0b100);
    assert_eq!(thread.flags, 0b1000);
    assert_eq!(Thread::flag_wait_all(0b111), 0b111);
    assert_eq!(thread.flags, 0b11000);
}
