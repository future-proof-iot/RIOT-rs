use embedded_threads::channel::Channel;

pub use crate::c::thread::thread_t;
pub use crate::thread::ThreadId;
pub use crate::thread::THREADS_NUMOF;

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

const EMPTY_CHANNEL: Channel<msg_t> = Channel::new();

static mut THREAD_CHANNELS: [Channel<msg_t>; THREADS_NUMOF as usize] =
    [EMPTY_CHANNEL; THREADS_NUMOF as usize];

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
    THREAD_CHANNELS[target_pid as usize].send(msg);
    1
}

#[no_mangle]
pub unsafe extern "C" fn msg_receive(msg: &mut msg_t) {
    *msg = THREAD_CHANNELS[super::thread::thread_getpid() as usize].recv();
}

#[no_mangle]
pub unsafe extern "C" fn msg_try_receive(msg: &mut msg_t) -> bool {
    if let Some(msg_) = THREAD_CHANNELS[super::thread::thread_getpid() as usize].try_recv() {
        *msg = msg_;
        true
    } else {
        false
    }
}

#[no_mangle]
pub unsafe extern "C" fn msg_send_receive(
    _msg: *mut msg_t,
    _reply: *mut msg_t,
    _target_pid: ThreadId,
) -> bool {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn msg_reply(_msg: &mut msg_t, _reply: &mut msg_t) -> i32 {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn msg_try_send(msg: &mut msg_t, target_pid: ThreadId) -> bool {
    THREAD_CHANNELS[target_pid as usize].try_send(msg)
}

#[no_mangle]
pub unsafe extern "C" fn msg_init_queue(_array: &'static mut msg_t, _num: usize) {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn msg_send_to_self(_msg: &mut msg_t) -> i32 {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn thread_has_msg_queue(_thread: &thread_t) -> bool {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn msg_avail() -> i32 {
    unimplemented!();
}
