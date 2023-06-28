#![no_std]
#![no_main]

use cortex_m_rt::entry;
use cortex_m_semihosting::{
    debug::{self, EXIT_SUCCESS},
    hprintln as println,
};

use panic_semihosting as _;

use embedded_threads::{self, channel::Channel, start_threading, thread_create};

static mut STACK: [u8; 4096] = [0; 4096];
static mut STACK2: [u8; 4096] = [0; 4096];
static mut STACK3: [u8; 4096] = [0; 4096];

static channel: Channel<[u8; 8]> = Channel::new();

fn test_thread1(_: ()) {
    let pid = embedded_threads::current_pid().unwrap();
    println!("test_thread() pid={}", pid);

    embedded_threads::schedule();

    println!("{}: getting msg", pid);
    let msg = channel.recv();

    println!("{}: got msg {:?}", pid, msg);

    println!("{}: done", pid);

    // testing asserts
    assert!(1 == 1);
}

fn test_thread2(_: ()) {
    let pid = embedded_threads::current_pid().unwrap();
    println!("test_thread() pid={}", pid);

    embedded_threads::schedule();

    println!("{}: sending msg", pid);
    let txt = b"foo bar!";
    channel.send(txt);

    println!("{}: sent msg", pid);
    println!("{}: sending msg", pid);
    let txt = b"foo bar!";
    channel.send(txt);

    println!("{}: sent msg", pid);
    println!("{}: done", pid);

    // testing asserts
    assert!(1 == 1);
}

#[entry]
fn main() -> ! {
    println!("main() creating thread 1");
    thread_create(test_thread1, (), unsafe { &mut STACK }, 1);

    println!("main() creating thread 2");
    thread_create(test_thread2, (), unsafe { &mut STACK2 }, 0);

    println!("main() creating thread 3");
    thread_create(test_thread1, (), unsafe { &mut STACK3 }, 0);

    println!("main() post thread create, starting threading");

    unsafe { start_threading() };

    // exit via semihosting call
    debug::exit(EXIT_SUCCESS);

    // the cortex_m_rt `entry` macro requires `main()` to never return
    loop {}
}
