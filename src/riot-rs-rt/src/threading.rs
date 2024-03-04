use riot_rs_threads::{start_threading, THREAD_FNS};

/// # Safety
///
/// The caller must ensure that this function is only called once.
///
/// # Panics
///
/// Panics if no thread exists.
pub unsafe fn start() -> ! {
    for thread_fn in THREAD_FNS {
        thread_fn();
    }

    // SAFETY: this function must only be called once, enforced by caller
    unsafe {
        start_threading();
    }

    loop {}
}
