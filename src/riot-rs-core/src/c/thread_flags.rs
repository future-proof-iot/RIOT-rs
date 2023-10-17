pub use super::thread::thread_t;
use super::thread::thread_t2id;
pub use crate::thread::{thread_flags, thread_flags::ThreadFlags};

pub const THREAD_FLAG_MSG_WAITING: ThreadFlags = 1 << 15;
pub const THREAD_FLAG_TIMEOUT: ThreadFlags = 1 << 14;

#[no_mangle]
pub unsafe extern "C" fn thread_flags_set(thread: &thread_t, mask: ThreadFlags) {
    thread_flags::set(thread_t2id(thread), mask)
}

#[no_mangle]
pub unsafe extern "C" fn thread_flags_wait_any(mask: ThreadFlags) -> ThreadFlags {
    thread_flags::wait_any(mask)
}

#[no_mangle]
pub unsafe extern "C" fn thread_flags_wait_one(mask: ThreadFlags) -> ThreadFlags {
    thread_flags::wait_one(mask)
}

#[no_mangle]
pub unsafe extern "C" fn thread_flags_wait_all(mask: ThreadFlags) -> ThreadFlags {
    thread_flags::wait_all(mask)
}

#[no_mangle]
pub unsafe extern "C" fn thread_flags_clear(mask: ThreadFlags) -> ThreadFlags {
    thread_flags::clear(mask)
}
