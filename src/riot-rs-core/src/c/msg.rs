pub use super::thread::thread_t;
pub use crate::ThreadId;

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
pub unsafe extern "C" fn thread_has_msg_queue(thread: &thread_t) -> bool {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn msg_avail() -> i32 {
    unimplemented!();
}
