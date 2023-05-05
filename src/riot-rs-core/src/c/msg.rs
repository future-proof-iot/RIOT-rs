use core::mem::MaybeUninit;

use crate::buffered_channel::BufferedChannel;

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

const EMPTY_CHANNEL: BufferedChannel<msg_t> = BufferedChannel::new();

static mut THREAD_CHANNELS: [BufferedChannel<msg_t>; THREADS_NUMOF as usize] =
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
    msg: *mut msg_t,
    reply: *mut msg_t,
    target_pid: ThreadId,
) -> bool {
    // C might hand over the same pointer as msg and reply....
    let reply_is_msg = msg == reply;

    let msg = &mut *msg;
    msg_send(msg, target_pid);

    let actual_reply = THREAD_CHANNELS[super::thread::thread_getpid() as usize].recv();

    if reply_is_msg {
        *msg = actual_reply;
    } else {
        let reply = &mut *reply;
        *reply = actual_reply;
    }

    true
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
pub unsafe extern "C" fn msg_init_queue(array: &'static mut msg_t, num: usize) {
    let queue: &'static mut MaybeUninit<msg_t> = core::mem::transmute(array);
    let queue = core::slice::from_raw_parts_mut(queue, num);
    THREAD_CHANNELS[super::thread::thread_getpid() as usize].set_backing_array(Some(queue))
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
    THREAD_CHANNELS[super::thread::thread_getpid() as usize].available() as i32
}
