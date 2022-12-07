use atomic_polyfill::{AtomicUsize, Ordering};
use core::arch::asm;
use core::cell::UnsafeCell;
use core::ptr::write_volatile;

use cortex_m::{
    interrupt::{self, CriticalSection},
    peripheral::SCB,
};

use bitflags::bitflags;
use cstr_core::c_char;

use clist::Link;
use riot_rs_runqueue::{RunQueue, RunqueueId, ThreadId};

/// global defining the number of possible priority levels
pub const SCHED_PRIO_LEVELS: usize = 16;

/// global defining the number of threads that can be created
pub const THREADS_NUMOF: usize = 12;

/// Flag that gets set whenever there's a message that can be received
pub const THREAD_FLAG_MSG_WAITING: ThreadFlags = (1 as ThreadFlags) << 15;

/// Flag to indicate timeout of some operations
pub const THREAD_FLAG_TIMEOUT: ThreadFlags = (1 as ThreadFlags) << 14;

/// type used as numerical ID for threads
pub type Pid = ThreadId;

/// type of thread flags
pub type ThreadFlags = u16;

/// Trait used to hide forced access to a global using a critical section
trait UncheckedMut<T> {
    fn unchecked_mut<'cs>(&self, cs: &'cs CriticalSection) -> &mut T;
}

impl<T> UncheckedMut<T> for UnsafeCell<T> {
    fn unchecked_mut<'cs>(&self, _cs: &'cs CriticalSection) -> &mut T {
        unsafe { self.get().as_mut().unwrap_unchecked() }
    }
}

/// Main struct for holding thread data
#[derive(Debug)]
pub struct Thread {
    sp: usize,
    high_regs: [usize; 8],
    pub(crate) list_entry: Link,
    pub(crate) state: ThreadState,
    pub prio: RunqueueId,
    pub flags: ThreadFlags,
    pub pid: Pid,

    #[cfg(feature = "thread_info")]
    name: Option<&'static c_char>,
    #[cfg(feature = "thread_info")]
    stack_bottom: usize,
    #[cfg(feature = "thread_info")]
    stack_size: usize,
}

/// Possible states of a thread
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ThreadState {
    Invalid,
    Running,
    Paused,
    Zombie,
    MutexBlocked,
    FlagBlocked(FlagWaitMode),
    ChannelRxBlocked(usize),
    ChannelTxBlocked(usize),
    ChannelReplyBlocked(usize),
    ChannelTxReplyBlocked(usize),
}

/// type used to add threads to lists (e.g., waiting for Lock, ...)
pub(crate) type ThreadList = clist::TypedList<Thread, { clist::offset_of!(Thread, list_entry) }>;

bitflags! {
    #[derive(Default)]
    #[repr(C)]
/// flags that can be set on thread creation
///
/// (Part of the C API)
    pub struct CreateFlags: u32 {
        const SLEEPING      = 0b00000001;
        const WITHOUT_YIELD = 0b00000010;
        const STACKTEST     = 0b00000100;
    }
}

/// Possible waiting modes for thread flags
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum FlagWaitMode {
    Any(ThreadFlags),
    All(ThreadFlags),
}

/// global thread runqueue
static mut RUNQUEUE: UnsafeCell<RunQueue<SCHED_PRIO_LEVELS, THREADS_NUMOF>> =
    UnsafeCell::new(RunQueue::new());

/// global thread list
static mut THREADS: [Thread; THREADS_NUMOF] = [const { Thread::default() }; THREADS_NUMOF];

/// global pointing to currently active thread
static CURRENT_THREAD: AtomicUsize = AtomicUsize::new(0);

/// global number of currently used (non-invalid) threads
#[no_mangle]
pub static THREADS_IN_USE: AtomicUsize = AtomicUsize::new(0);

// RIOT's power management hook
extern "C" {
    fn pm_set_lowest();
}

/// scheduler
#[no_mangle]
pub unsafe fn sched(old_sp: usize) {
    let mut current = Thread::current();

    let next_pid;

    loop {
        if let Some(pid) = RUNQUEUE.unchecked_mut(&CriticalSection::new()).get_next() {
            next_pid = pid;
            break;
        }
        pm_set_lowest();
        cortex_m::interrupt::enable();
        // pending interrupts would now get to run their ISRs
        cortex_m::interrupt::disable();
    }

    let next = Thread::get(next_pid);

    if next as *const Thread == current as *const Thread {
        asm!("", in("r0") 0);
        return;
    }

    current.sp = old_sp;
    CURRENT_THREAD.store((next as *const Thread) as usize, Ordering::Release);

    // PendSV expects these three pointers in r0, r1 and r2
    // write to registers manually, as ABI would return the values via stack
    asm!("", in("r0") current.high_regs.as_ptr(), in("r1") next.high_regs.as_ptr(), in("r2")next.sp);
}

/// thread cleanup function
///
/// This gets hooked into a newly created thread stack so it gets called when
/// the thread function returns.
pub fn cleanup() -> ! {
    let current = Thread::current();
    //hprintln!("thread {} ended.", current.pid);
    current.set_state(ThreadState::Invalid);
    THREADS_IN_USE.fetch_sub(1, Ordering::SeqCst);
    Thread::yield_higher();

    loop {}
}

/// Supervisor Call exception handler
///
/// This is currently used to initiate threading.
#[cfg(any(armv7m))]
#[naked]
#[no_mangle]
#[allow(non_snake_case)]
unsafe extern "C" fn SVCall() {
    asm!(
        "
            movw LR, #0xFFFd
            movt LR, #0xFFFF
            bx lr
            ",
        options(noreturn)
    );
}

#[cfg(any(armv6m))]
#[naked]
#[no_mangle]
#[allow(non_snake_case)]
unsafe extern "C" fn SVCall() {
    asm!(
        "
            /* label rules:
             * - number only
             * - no combination of *only* [01]
             * - add f or b for 'next matching forward/backward'
             * so let's use '99' forward ('99f')
             */
            ldr r0, 99f
            mov LR, r0
            bx lr

            .align 4
            99:
            .word 0xFFFFFFFD
            ",
        options(noreturn)
    );
}

/// PendSV exception handler
#[cfg(any(armv7m))]
#[naked]
#[no_mangle]
#[allow(non_snake_case)]
unsafe extern "C" fn PendSV() {
    asm!(
        "
            mrs r0, psp
            cpsid i
            bl {sched}
            cpsie i
            cmp r0, #0
            /* label rules:
             * - number only
             * - no combination of *only* [01]
             * - add f or b for 'next matching forward/backward'
             * so let's use '99' forward ('99f')
             */
            beq 99f
            stmia r0, {{r4-r11}}
            ldmia r1, {{r4-r11}}
            msr.n psp, r2
            99:
            movw LR, #0xFFFd
            movt LR, #0xFFFF
            bx LR
            ",
        sched = sym sched,
        options(noreturn)
    );
}

#[cfg(any(armv6m))]
#[naked]
#[no_mangle]
#[allow(non_snake_case)]
unsafe extern "C" fn PendSV() {
    asm!(
        "
            mrs r0, psp
            cpsid i
            bl sched
            cpsie i
            cmp r0, #0
            beq 99f

            //stmia r0!, {{r4-r7}}
            str r4, [r0, #16]
            str r5, [r0, #20]
            str r6, [r0, #24]
            str r7, [r0, #28]

            mov  r4, r8
            mov  r5, r9
            mov  r6, r10
            mov  r7, r11

            str r4, [r0, #0]
            str r5, [r0, #4]
            str r6, [r0, #8]
            str r7, [r0, #12]

            //
            ldmia r1!, {{r4-r7}}
            mov r11, r7
            mov r10, r6
            mov r9,  r5
            mov r8,  r4
            ldmia r1!, {{r4-r7}}

            msr.n psp, r2
            99:
            ldr r0, 999f
            mov LR, r0
            bx lr

            .align 4
            999:
            .word 0xFFFFFFFD
            ",
        options(noreturn)
    );
}

impl Thread {
    /// create a default Thread object
    ///
    pub const fn default() -> Thread {
        Thread {
            sp: 0,
            state: ThreadState::Invalid,
            list_entry: Link::new(),
            high_regs: [0; 8],
            prio: 0,
            pid: 0,
            flags: 0,
            #[cfg(feature = "thread_info")]
            name: None,
            #[cfg(feature = "thread_info")]
            stack_size: 0,
            #[cfg(feature = "thread_info")]
            stack_bottom: 0,
        }
    }

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
    fn create_(
        stack: &mut [u8],
        func: usize,
        arg: usize,
        prio: u8,
        flags: CreateFlags,
    ) -> &mut Thread {
        unsafe {
            let unused_pid = Thread::find_unused_pid().unwrap();
            let mut thread = &mut THREADS[unused_pid as usize];
            thread.sp = Thread::setup_stack(stack, func, arg);
            thread.pid = unused_pid;
            thread.prio = prio;

            THREADS_IN_USE.fetch_add(1, Ordering::SeqCst);

            if flags.contains(CreateFlags::SLEEPING) {
                thread.state = ThreadState::Paused;
            } else {
                thread.state = ThreadState::Running;
                interrupt::free(|cs| RUNQUEUE.unchecked_mut(cs).add(unused_pid, thread.prio));
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

    pub fn pid_is_valid(pid: Pid) -> bool {
        if pid as usize >= unsafe { THREADS.len() } {
            false
        } else {
            let thread = unsafe { &THREADS[pid as usize] };
            match thread.state {
                ThreadState::Invalid => false,
                _ => true,
            }
        }
    }

    pub fn current() -> &'static mut Thread {
        unsafe {
            return &mut *(CURRENT_THREAD.load(Ordering::Acquire) as *mut Thread);
        }
    }

    pub fn current_pid() -> Pid {
        Thread::current().pid
    }

    /// set state of thread
    ///
    /// This function handles adding/removing the thread to the Runqueue depending
    /// on its previous or new state.
    pub fn set_state(&mut self, state: ThreadState) {
        interrupt::free(|cs| {
            let old_state = self.state;
            self.state = state;
            if old_state != ThreadState::Running && state == ThreadState::Running {
                unsafe { RUNQUEUE.unchecked_mut(cs).add(self.pid, self.prio) };
            } else if old_state == ThreadState::Running && state != ThreadState::Running {
                unsafe { RUNQUEUE.unchecked_mut(cs).del(self.pid, self.prio) };
            }
        });
    }

    pub unsafe fn jump_to(&self) {
        CURRENT_THREAD.store((self as *const Thread) as usize, Ordering::Release);
        asm!(
            "
            msr psp, r1
            svc 0
            ",
        in("r1")self.sp);
    }

    /// "yields" to other threads of the same priority, if any
    pub fn yield_next() {
        let current = Thread::current();
        unsafe {
            interrupt::free(|cs| RUNQUEUE.unchecked_mut(cs).advance(current.prio));
            SCB::set_pendsv();
            cortex_m::asm::isb();
        }
    }

    /// triggers the scheduler
    pub fn yield_higher() {
        SCB::set_pendsv();
        cortex_m::asm::isb();
    }

    /// shortly enable interrupts
    ///
    /// Supposed to be called within a critical section to allow interrupts.
    pub fn isr_enable_disable() {
        core::sync::atomic::compiler_fence(Ordering::SeqCst);
        unsafe { cortex_m::interrupt::enable() };
        cortex_m::asm::isb();
        cortex_m::interrupt::disable();
    }

    /// Make thread "sleep"
    ///
    /// 1. sets thread state to "Paused"
    /// 2. triggers scheduler
    pub fn sleep() {
        Thread::current().set_state(ThreadState::Paused);
        Thread::yield_higher();
    }

    /// Wakes up a sleeping thread
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

    /// Put thread in waitlist with given state
    pub(crate) fn wait_on(&mut self, list: &mut ThreadList, wait_state: ThreadState) {
        list.rpush(self);
        self.set_state(wait_state);
        Thread::yield_higher();
    }

    // Zombify thread
    pub fn zombify(&mut self) {
        self.set_state(ThreadState::Zombie);
        Thread::yield_higher();
    }

    pub fn zombie_kill(&mut self) -> i32 {
        match self.state {
            ThreadState::Zombie => {
                self.set_state(ThreadState::Invalid);
                THREADS_IN_USE.fetch_sub(1, Ordering::SeqCst);
                1
            }
            _ => -1,
        }
    }

    /// Start riot-rs-core scheduler
    ///
    /// Note: this _must only be called once during startup_.
    pub unsafe fn start_threading() -> ! {
        let next_pid =
            interrupt::free(|cs| RUNQUEUE.unchecked_mut(cs).get_next().unwrap_unchecked() as Pid);
        Thread::get(next_pid).jump_to();
        loop {}
    }
}

impl Thread {
    // thread flags implementation
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

#[cfg(feature = "thread_info")]
impl Thread {
    pub fn name(&self) -> Option<&'static c_char> {
        self.name
    }

    pub fn stack_size(&self) -> usize {
        self.stack_size
    }

    pub fn stack_bottom(&self) -> *const u8 {
        self.stack_bottom as *const u8
    }

    pub fn stack_top(&self) -> *const u8 {
        ((self.stack_bottom as usize) + self.stack_size()) as *const u8
    }

    fn stack_canary_fill(stack: &[u8]) {
        let start_addr = stack.as_ptr() as usize;
        for i in 0..(stack.len() / 4) {
            let pos = start_addr + (i * 4);
            unsafe { *(pos as *mut usize) = pos };
        }
    }
}

#[cfg(not(feature = "thread_info"))]
impl Thread {
    pub fn name(&self) -> Option<&'static c_char> {
        None
    }

    pub fn stack_size(&self) -> usize {
        0
    }

    pub fn stack_bottom(&self) -> *const u8 {
        0 as *const u8
    }

    pub fn stack_top(&self) -> *const u8 {
        0 as *const u8
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
            interrupt::free(|cs| RUNQUEUE.unchecked_mut(cs).add(unused_pid, thread.prio));

            THREADS_IN_USE.fetch_add(1, Ordering::SeqCst);

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

/// C bindings and glue code
pub mod c {
    #![allow(non_camel_case_types)]
    use atomic_polyfill::{AtomicBool, Ordering};
    use core::ffi::c_void;

    use ref_cast::RefCast;
    use riot_rs_runqueue::RunqueueId;

    use crate::lock::Lock;
    use crate::thread::{CreateFlags, FlagWaitMode, Pid, Thread, ThreadFlags, ThreadState};

    use crate::thread::c_char;

    #[derive(RefCast)]
    #[repr(transparent)]
    pub struct thread_t(Thread);

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
        // println!(
        //     "stack_ptr as u8: {:#x} size: {}",
        //     stack_ptr as usize, stack_size
        // );

        // align end of stack (lowest address)
        let misalign = stack_ptr as usize & 0x7;
        let mut stack_ptr = stack_ptr;
        let mut stack_size = stack_size;
        if misalign > 0 {
            stack_ptr = (stack_ptr as usize + 8 - misalign) as *mut u8;
            stack_size -= 8 - misalign;
        }

        // align start of stack (lowest address plus stack_size)
        stack_size &= !0x7;

        // println!(
        //     "aligned stack_ptr as u8: {:#x} size: {}",
        //     stack_ptr as usize, stack_size
        // );

        let stack = core::slice::from_raw_parts_mut(stack_ptr, stack_size);

        #[cfg(feature = "thread_info")]
        Thread::stack_canary_fill(&stack);

        #[allow(unused_mut)] // (mut not needed without "thread_info" feature)
        let mut thread = Thread::create_(
            stack,
            thread_func,
            arg,
            priority,
            CreateFlags::from_bits_truncate(flags),
        );

        #[cfg(feature = "thread_info")]
        {
            thread.name = if _name as *const c_char as usize != 0 {
                Some(&*_name)
            } else {
                None
            };
            thread.stack_size = stack_size;
            thread.stack_bottom = stack_ptr as usize;
        }

        thread.pid
    }

    #[no_mangle]
    pub extern "C" fn thread_get_active() -> &'static mut thread_t {
        thread_t::ref_cast_mut(Thread::current())
    }

    #[no_mangle]
    pub unsafe extern "C" fn thread_get(pid: Pid) -> *mut thread_t {
        if Thread::pid_is_valid(pid) {
            thread_t::ref_cast_mut(Thread::get_mut(pid)) as *mut thread_t
        } else {
            core::ptr::null_mut()
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn thread_wakeup(pid: Pid) {
        Thread::wakeup(pid)
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
        core::arch::asm!("cpsie i");
        Thread::start_threading()
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
        cortex_m::interrupt::free(|_| {
            mutex.release();
            Thread::sleep();
        });
    }

    #[repr(C)]
    pub struct mutex_cancel_t {
        lock: &'static Lock,
        pid: Pid,
        cancelled: AtomicBool,
    }

    #[no_mangle]
    pub extern "C" fn mutex_cancel_init(mutex: &'static Lock) -> mutex_cancel_t {
        mutex_cancel_t {
            lock: mutex,
            pid: Thread::current_pid(),
            cancelled: AtomicBool::new(false),
        }
    }

    #[no_mangle]
    pub extern "C" fn mutex_cancel(mutex_cancel: &mutex_cancel_t) {
        if !AtomicBool::swap(&mutex_cancel.cancelled, true, Ordering::SeqCst) {
            unsafe { Thread::get_mut(mutex_cancel.pid).set_state(super::ThreadState::Running) };
        }
    }

    #[no_mangle]
    pub extern "C" fn mutex_lock_cancelable(mutex_cancel: &mutex_cancel_t) -> i32 {
        let early_cancel = cortex_m::interrupt::free(|_| {
            if AtomicBool::load(&mutex_cancel.cancelled, Ordering::SeqCst) {
                return 2;
            } else if mutex_cancel.lock.try_acquire() {
                return 1;
            } else {
                mutex_cancel.lock.acquire();
                return 0;
            }
        });

        match early_cancel {
            2 => -140,
            1 => 0,
            _ => {
                cortex_m::interrupt::free(|cs| {
                    // if we end up here, the mutex was neither cancelled early,
                    // nor did try_acquire() get it.
                    if mutex_cancel.lock.fix_cancelled_acquire(cs) {
                        // we were removed from the waiting list.
                        // that means the acquire was cancelled.
                        // return -ECANCELED (-140 in newlib)
                        // TODO: use proper constant
                        -140
                    } else {
                        // we were not removed from the waiting list.
                        // that means we got the mutex in the meantime,
                        // or we were the thread locking the mutex (it was locked,
                        // but with an empty waiting list).
                        if AtomicBool::load(&mutex_cancel.cancelled, Ordering::SeqCst) {
                            -140
                        } else {
                            0
                        }
                    }
                })
            }
        }
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
        pub sender_pid: Pid,
        pub type_: u16,
        pub content: msg_content_t,
    }

    impl Default for msg_t {
        fn default() -> Self {
            msg_t {
                sender_pid: Pid::default(),
                type_: 0u16,
                content: msg_content_t { value: 0 },
            }
        }
    }

    use crate::channel::BufferedChannel;

    // "const {...}" in array initializer causes ICE on rustc 1.53, so use temporary
    const TMP: BufferedChannel<msg_t> = BufferedChannel::new(None);
    static mut THREAD_MSG_CHANNELS: [BufferedChannel<msg_t>; super::THREADS_NUMOF] =
        [TMP; super::THREADS_NUMOF];

    pub(crate) fn get_channel_for_pid(pid: Pid) -> &'static mut BufferedChannel<'static, msg_t> {
        unsafe { &mut THREAD_MSG_CHANNELS[pid as usize] }
    }

    #[no_mangle]
    pub unsafe extern "C" fn msg_send(msg: &mut msg_t, target_pid: Pid) -> i32 {
        // TODO: handle nonexisting Pid
        msg.sender_pid = Thread::current_pid();
        if msg.sender_pid == target_pid {
            return msg_send_to_self(msg);
        }
        let target = get_channel_for_pid(target_pid);
        target.send(*msg);
        thread_flags_set(Thread::get_mut(target_pid), super::THREAD_FLAG_MSG_WAITING);
        1
    }

    #[no_mangle]
    pub unsafe extern "C" fn msg_receive(msg: &mut msg_t) {
        let channel = get_channel_for_pid(Thread::current_pid());
        *msg = channel.recv();
    }

    #[no_mangle]
    pub unsafe extern "C" fn msg_try_receive(msg: &mut msg_t) -> bool {
        let channel = get_channel_for_pid(Thread::current_pid());
        match channel.try_recv() {
            Ok(res) => {
                *msg = res;
                true
            }
            _ => false,
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn msg_send_receive(
        msg: *mut msg_t,
        reply: *mut msg_t,
        target_pid: Pid,
    ) -> bool {
        // C might hand over the same pointer as msg and reply....
        let reply_is_msg = msg == reply;

        let mut msg = &mut *msg;
        msg.sender_pid = Thread::current_pid();
        if msg.sender_pid == target_pid {
            return false;
        }

        let target = get_channel_for_pid(target_pid);
        let actual_reply = target.send_reply(*msg, target_pid);

        if reply_is_msg {
            *msg = actual_reply;
        } else {
            let reply = &mut *reply;
            *reply = actual_reply;
        }

        true
    }

    #[no_mangle]
    pub unsafe extern "C" fn msg_reply(msg: &mut msg_t, reply: &mut msg_t) -> i32 {
        cortex_m::interrupt::free(|_| {
            let target = Thread::get_mut(msg.sender_pid);
            if let ThreadState::ChannelReplyBlocked(ptr) = target.state {
                core::ptr::write_volatile(ptr as *mut msg_t, *reply);
                target.set_state(ThreadState::Running);
                Thread::yield_higher();
                1
            } else {
                -1
            }
        })
    }

    #[no_mangle]
    pub unsafe extern "C" fn msg_try_send(msg: &mut msg_t, target_pid: Pid) -> bool {
        msg.sender_pid = Thread::current_pid();
        let channel = get_channel_for_pid(target_pid);
        match channel.try_send(*msg) {
            Ok(()) => {
                thread_flags_set(Thread::get_mut(target_pid), super::THREAD_FLAG_MSG_WAITING);
                true
            }
            _ => false,
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn msg_init_queue(array: &'static mut msg_t, num: usize) {
        let channel = get_channel_for_pid(Thread::current_pid());
        channel.set_backing_array(Some(core::slice::from_raw_parts_mut(array, num)));
    }

    #[no_mangle]
    pub unsafe extern "C" fn msg_send_to_self(msg: &mut msg_t) -> i32 {
        if msg_try_send(msg, thread_getpid()) {
            1
        } else {
            0
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn thread_has_msg_queue(thread: &Thread) -> bool {
        get_channel_for_pid(thread.pid).capacity() > 0
    }

    #[no_mangle]
    pub unsafe extern "C" fn msg_avail() -> i32 {
        get_channel_for_pid(Thread::current_pid()).available() as i32
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

    #[no_mangle]
    pub unsafe extern "C" fn thread_zombify() {
        cortex_m::interrupt::free(|_| {
            let current = Thread::current();
            current.zombify();
        });
    }

    #[no_mangle]
    pub unsafe extern "C" fn thread_kill_zombie(pid: Pid) -> i32 {
        cortex_m::interrupt::free(|_| {
            if Thread::pid_is_valid(pid) {
                return -1; /* STATUS_NOT_FOUND */
            }

            let thread = Thread::get_mut(pid);
            thread.zombie_kill()
        })
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
                ThreadState::MutexBlocked => thread_status_t::MutexBlocked,
                ThreadState::FlagBlocked(FlagWaitMode::Any(_)) => thread_status_t::FlagBlockedAny,
                ThreadState::FlagBlocked(FlagWaitMode::All(_)) => thread_status_t::FlagBlockedAll,
                ThreadState::ChannelRxBlocked(_) => thread_status_t::ChannelRxBlocked,
                ThreadState::ChannelTxBlocked(_) => thread_status_t::ChannelTxBlocked,
                ThreadState::ChannelReplyBlocked(_) => thread_status_t::ChannelReplyBlocked,
                ThreadState::ChannelTxReplyBlocked(_) => thread_status_t::ChannelTxReplyBlocked,
            }
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn thread_state_to_string(state: thread_status_t) -> *const c_char {
        let res = match state {
            thread_status_t::Running => b"pending\0".as_ptr(),
            thread_status_t::Zombie => b"zombie\0".as_ptr(),
            thread_status_t::Paused => b"sleeping\0".as_ptr(),
            thread_status_t::MutexBlocked => b"bl mutex\0".as_ptr(),
            thread_status_t::FlagBlockedAny => b"bl anyfl\0".as_ptr(),
            thread_status_t::FlagBlockedAll => b"bl allfl\0".as_ptr(),
            thread_status_t::ChannelTxBlocked => b"bl send\0".as_ptr(),
            thread_status_t::ChannelRxBlocked => b"bl rx\0".as_ptr(),
            thread_status_t::ChannelTxReplyBlocked => b"bl txrx\0".as_ptr(),
            thread_status_t::ChannelReplyBlocked => b"bl reply\0".as_ptr(),
            _ => b"unknown\0".as_ptr(),
        };
        res as *const u8 as usize as *const c_char
    }

    #[no_mangle]
    pub unsafe extern "C" fn thread_get_status(thread: &Thread) -> thread_status_t {
        thread_status_t::from(thread.state)
    }

    #[no_mangle]
    pub unsafe extern "C" fn thread_getpid_of(thread: &Thread) -> Pid {
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
    pub unsafe extern "C" fn thread_get_stackstart(thread: &Thread) -> *const c_void {
        thread.stack_bottom() as *const c_void
    }

    #[no_mangle]
    pub unsafe extern "C" fn thread_get_stacksize(thread: &Thread) -> usize {
        thread.stack_size()
    }

    #[no_mangle]
    pub unsafe extern "C" fn thread_get_name(thread: &Thread) -> *const c_char {
        if let Some(name) = thread.name() {
            name as *const c_char
        } else {
            0 as *const c_char
        }
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
fn test_pid_is_zero() {
    // note: this is true if there's no idle thread
    assert!(Thread::current().pid == 0);
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
