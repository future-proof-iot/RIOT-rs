#![no_std]
#![no_main]

use cortex_m_rt::entry;
use cortex_m_semihosting::{
    debug::{self, ExitCode},
    hprintln as println,
};

use panic_semihosting as _;

use embedded_threads::{self, start_threading, thread_create};

static mut STACK: [u8; 2048] = [0; 2048];
static mut STACK2: [u8; 2048] = [0; 2048];

fn test_thread(arg: usize) {
    println!("test_thread() arg={}", arg);

    // testing asserts
    assert!(1 == 1);
}

#[entry]
fn main() -> ! {
    println!("main() creating thread 1");
    thread_create(test_thread, 1, unsafe { &mut STACK }, 0);

    println!("main() creating thread 2");
    thread_create(test_thread, 2, unsafe { &mut STACK2 }, 1);

    println!("main() post thread create, starting threading");

    unsafe { start_threading() };

    println!("main() shouldn't be here");
    // exit via semihosting call
    debug::exit(ExitCode::SUCCESS);

    // the cortex_m_rt `entry` macro requires `main()` to never return
    loop {}
}
