#![no_std]
#![no_main]

use cortex_m_rt::entry;
use cortex_m_semihosting::{
    debug::{self, EXIT_SUCCESS},
    hprintln as println,
};

use panic_semihosting as _;

use embedded_threads::{self, start_threading, thread_create, thread_flags::*};

static mut STACK: [u8; 2048] = [0; 2048];
static mut STACK2: [u8; 2048] = [0; 2048];
static mut STACK3: [u8; 2048] = [0; 2048];
static mut STACK4: [u8; 2048] = [0; 2048];

fn test_thread(_: ()) {
    let pid = embedded_threads::current_pid().unwrap();
    println!("{}: test_thread() started", pid);

    set(1, 1);
    set(2, 1);
    set(3, 1);

    println!("{}: test_thread() finished", pid);
}

fn test_thread_flag_waiter_any(arg: usize) {
    let pid = embedded_threads::current_pid().unwrap();
    println!("{}: test_thread_flag_waiter_any() arg={}", pid, arg);

    wait_any(arg as ThreadFlags);

    println!("{}: test_thread_flag_waiter_any() finished", pid);
}

#[entry]
fn main() -> ! {
    println!("main() creating thread 1");
    thread_create(test_thread, (), unsafe { &mut STACK }, 0);

    println!("main() creating thread 2");
    thread_create(test_thread_flag_waiter_any, 1, unsafe { &mut STACK2 }, 1);

    println!("main() creating thread 3");
    thread_create(test_thread_flag_waiter_any, 2, unsafe { &mut STACK3 }, 1);

    println!("main() creating thread 4");
    thread_create(test_thread_flag_waiter_any, 3, unsafe { &mut STACK4 }, 1);

    println!("main() post thread create, starting threading");

    unsafe { start_threading() };

    println!("main() shouldn't be here");
    // exit via semihosting call
    debug::exit(EXIT_SUCCESS);

    // the cortex_m_rt `entry` macro requires `main()` to never return
    loop {}
}
