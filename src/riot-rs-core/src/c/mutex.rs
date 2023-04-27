pub use crate::thread::{self, lock::Lock, Thread, ThreadId};

// cbindgen cannot export these
//pub const MUTEX_T_SIZEOF: usize = core::mem::sizeof::<Lock>();
//pub const MUTEX_T_ALIGNOF: usize = core::mem::align_of::<Lock>();
// TODO: static_assert these, and provide per-arch variants
pub const MUTEX_T_SIZEOF: usize = 2;
pub const MUTEX_T_ALIGNOF: usize = 1;

unsafe fn ensure_initialized(mutex: &mut mutex_t) {
    critical_section::with(|_| {
        if mutex.bytes == [0x00, 0x00] {
            mutex_init(mutex);
        } else if mutex.bytes == [0xff, 0xff] {
            mutex_init_locked(mutex);
        }
    })
}

/// cbindgen:ignore
#[allow(non_camel_case_types)]
pub union mutex_t {
    bytes: [u8; core::mem::size_of::<Lock>()],
    lock: core::mem::ManuallyDrop<Lock>,
}

#[no_mangle]
pub unsafe extern "C" fn mutex_init(mutex: &mut mutex_t) {
    *mutex.lock = Lock::new()
}

#[no_mangle]
pub unsafe extern "C" fn mutex_init_locked(mutex: &mut mutex_t) {
    *mutex.lock = Lock::new_locked()
}

#[no_mangle]
pub unsafe extern "C" fn mutex_lock(mutex: &mut mutex_t) {
    ensure_initialized(mutex);
    mutex.lock.acquire()
}

#[no_mangle]
pub unsafe extern "C" fn mutex_trylock(mutex: &mut mutex_t) -> bool {
    ensure_initialized(mutex);
    mutex.lock.try_acquire()
}

#[no_mangle]
pub unsafe extern "C" fn mutex_unlock(mutex: &mut mutex_t) {
    ensure_initialized(mutex);
    mutex.lock.release()
}

#[no_mangle]
pub unsafe extern "C" fn mutex_unlock_and_sleep(mutex: &mut mutex_t) {
    critical_section::with(|_| {
        mutex_unlock(mutex);
        thread::sleep();
    });
}

#[repr(C)]
pub struct mutex_cancel_t {
    lock: &'static Lock,
    pid: ThreadId,
    //        cancelled: AtomicBool,
}

#[no_mangle]
pub extern "C" fn mutex_cancel_init(_mutex: &'static mutex_t) -> mutex_cancel_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn mutex_cancel(_mutex_cancel: &mutex_cancel_t) {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn mutex_lock_cancelable(_mutex_cancel: &mutex_cancel_t) -> i32 {
    unimplemented!();
}
