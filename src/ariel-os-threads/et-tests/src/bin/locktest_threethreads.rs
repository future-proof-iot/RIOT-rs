#![no_std]
#![no_main]

use cortex_m_rt::entry;
use cortex_m_semihosting::{
    debug::{self, ExitCode},
    hprintln as println,
};

use panic_semihosting as _;

use embedded_threads::{self, lock::Lock, start_threading, thread_create};

static mut STACK: [u8; 4096] = [0; 4096];
static mut STACK2: [u8; 4096] = [0; 4096];
static mut STACK3: [u8; 4096] = [0; 4096];

fn test_thread(lock: &Lock) {
    let pid = embedded_threads::current_pid().unwrap();
    println!("test_thread() pid={}", pid);

    println!("{}: lock state: {}", pid, lock.is_locked());

    if lock.is_locked() {
        println!("{}: releasing lock", pid);
        lock.release();
    } else {
        println!("{}: getting lock (a)", pid);

        lock.acquire();
    }

    println!("{}: getting lock (b)", pid);

    lock.acquire();

    println!("{}: releasing lock (b)", pid);
    lock.release();

    println!("{}: done", pid);

    // testing asserts
    assert!(1 == 1);
}

fn test_thread2_3(lock: &Lock) {
    let pid = embedded_threads::current_pid().unwrap();
    println!("test_thread() pid={}", pid);

    println!("{}: lock state: {}", pid, lock.is_locked());

    println!("{}: getting lock (a)", pid);
    lock.acquire();

    println!("{}: releasing lock", pid);
    lock.release();

    println!("{}: done", pid);

    // testing asserts
    assert!(1 == 1);
}

#[entry]
fn main() -> ! {
    let lock = Lock::new_locked();

    println!("main() creating thread 1");
    thread_create(test_thread, &lock, unsafe { &mut STACK }, 0);

    println!("main() creating thread 2");
    thread_create(test_thread2_3, &lock, unsafe { &mut STACK2 }, 1);

    println!("main() creating thread 3");
    thread_create(test_thread2_3, &lock, unsafe { &mut STACK3 }, 2);

    println!("main() post thread create, starting threading");

    unsafe { start_threading() };

    println!("main() shouldn't be here");
    // exit via semihosting call
    debug::exit(ExitCode::SUCCESS);

    // the cortex_m_rt `entry` macro requires `main()` to never return
    loop {}
}
