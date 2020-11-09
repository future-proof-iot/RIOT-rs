use crate::thread::{CreateFlags, Thread};

static mut IDLE_STACK: [u8; 256] = [0; 256];
static mut MAIN_STACK: [u8; 2048] = [0; 2048];

fn idle(_arg: usize) {
    loop {
        cortex_m::asm::wfi();
    }
}

extern "C" {
    fn user_main();
}

fn main_trampoline(_arg: usize) {
    unsafe {
        user_main();
    }
}

pub fn startup() {
    unsafe {
        Thread::create(&mut IDLE_STACK, idle, 0, 0, CreateFlags::WITHOUT_YIELD);
        Thread::create(
            &mut MAIN_STACK,
            main_trampoline,
            1,
            5,
            CreateFlags::WITHOUT_YIELD,
        )
        .jump_to();
    }
}
