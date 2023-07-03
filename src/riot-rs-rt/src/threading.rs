use riot_rs_threads::{start_threading, thread_create};

static mut MAIN_STACK: [u8; 2048] = [0; 2048];

extern "Rust" {
    fn riot_main();
}

fn main_trampoline(_arg: usize) {
    unsafe {
        riot_main();
    }
}

pub(crate) fn init() -> ! {
    unsafe {
        thread_create(main_trampoline, 0, &mut MAIN_STACK, 0);
        start_threading();
    }
    loop {}
}
