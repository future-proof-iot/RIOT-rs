use riot_rs_threads::{start_threading, thread_create, THREAD_FNS};

const MAIN_STACK_SIZE: usize = 2048;

extern "Rust" {
    fn riot_main();
}

fn main_trampoline(_arg: usize) {
    // SAFETY: FFI call to a Rust function
    unsafe {
        riot_main();
    }
}

/// # Safety
///
/// The caller must ensure that this function is only called once.
pub unsafe fn start() -> ! {
    for thread_fn in THREAD_FNS {
        thread_fn();
    }

    let mut main_stack: [u8; MAIN_STACK_SIZE] = [0; MAIN_STACK_SIZE];
    thread_create(main_trampoline, 0, &mut main_stack, 0);

    // SAFETY: this function must only be called once, enforced by caller
    unsafe {
        start_threading();
    }

    loop {}
}
