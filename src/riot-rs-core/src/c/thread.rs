use core::ffi::{c_char, c_void};
use core::unimplemented;
use critical_section::{self};

pub use embedded_threads::{RunqueueId, Thread, ThreadFlags, ThreadId, ThreadState, WaitMode};
pub use ref_cast::RefCast;

use embedded_threads::current_pid;

#[allow(non_camel_case_types)]
pub type thread_t = c_void;

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
    let stack_ptr = stack_ptr as *mut c_char as usize as *mut u8;
    // // println!(
    // //     "stack_ptr as u8: {:#x} size: {}",
    // //     stack_ptr as usize, stack_size
    // // );

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

    let stack = core::slice::from_raw_parts_mut(stack_ptr, stack_size);

    embedded_threads::thread_create_raw(thread_func, arg, stack, priority)
}

#[no_mangle]
pub extern "C" fn thread_get_active() -> *mut thread_t {
    if let Some(thread_id) = current_pid() {
        thread_id as usize as *mut thread_t
    } else {
        core::ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn thread_get(thread_id: ThreadId) -> *mut thread_t {
    if embedded_threads::is_valid_pid(thread_id) {
        thread_id as usize as *mut thread_t
    } else {
        core::ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn thread_wakeup(pid: ThreadId) {
    unimplemented!();
    // Thread::wakeup(pid)
}

#[no_mangle]
pub extern "C" fn thread_yield_higher() {
    embedded_threads::schedule();
}

#[no_mangle]
pub extern "C" fn thread_yield() {
    embedded_threads::yield_same();
}

#[no_mangle]
pub extern "C" fn thread_getpid() -> ThreadId {
    unimplemented!();
    // Thread::current_pid()
    0
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
            ThreadState::LockBlocked => thread_status_t::MutexBlocked,
            ThreadState::FlagBlocked(WaitMode::Any(_)) => thread_status_t::FlagBlockedAny,
            ThreadState::FlagBlocked(WaitMode::All(_)) => thread_status_t::FlagBlockedAll,
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
    //thread.stack_bottom() as *mut c_void
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn thread_get_stacksize(thread: &Thread) -> usize {
    //thread.stack_size()
    0
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
