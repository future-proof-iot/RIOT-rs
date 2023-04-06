pub use crate::Lock;
pub use crate::ThreadId;
pub use embedded_threads::thread::Thread;

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
        embedded_threads::thread::sleep();
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
