pub use super::thread::thread_t;
pub use embedded_threads::ThreadFlags;

#[no_mangle]
pub unsafe extern "C" fn thread_flags_set(thread: &thread_t, mask: ThreadFlags) {
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
