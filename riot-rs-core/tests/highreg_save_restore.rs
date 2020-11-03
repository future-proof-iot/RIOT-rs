#![no_main]
#![no_std]
// testing
#![feature(custom_test_frameworks)]
#![test_runner(riot_rs_rt::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(llvm_asm)]

use riot_rs_core::thread::{CreateFlags, Thread};
use riot_rs_rt as _;

#[no_mangle]
extern "C" fn user_main() {
    #[cfg(test)]
    test_main();
}

fn func(_arg: usize) {
    loop {
        unsafe {
            llvm_asm!("
        mov r4, #4
        mov r5, #5
        mov r6, #6
        #mov r7, #7
        mov r8, #8
        mov r9, #9
        mov r10, #10
        mov r11, #11
        " :::: "volatile");
        }
        Thread::yield_next();
    }
}

static mut STACK: [u8; 2048] = [0; 2048];

#[test_case]
fn test_hireg_save_restore() {
    unsafe {
        Thread::create(&mut STACK, func, 0, 5, CreateFlags::empty());
    }
    unsafe {
        llvm_asm!("
        mov r4, #0x44
        mov r5, #0x55
        mov r6, #0x66
        //mov r7, #0x77
        mov r8, #0x88
        mov r9, #0x99
        mov r10, #0x1010
        mov r11, #0x1111
        " :::: "volatile");
    }
    Thread::yield_next();
    let r4: usize;
    let r5: usize;
    let r6: usize;
    //let r7: usize;
    let r8: usize;
    let r9: usize;
    let r10: usize;
    let r11: usize;

    unsafe {
        llvm_asm!(""
                :
                "={r4}"(r4),
                "={r5}"(r5),
                "={r6}"(r6),
                //"={r7}"(r7),
                "={r8}"(r8),
                "={r9}"(r9),
                "={r10}"(r10),
                "={r11}"(r11)
                ::: "volatile");
    }
    assert!(r4 == 0x44);
    assert!(r5 == 0x55);
    assert!(r6 == 0x66);
    //assert!(r7 == 0x77);
    assert!(r8 == 0x88);
    assert!(r9 == 0x99);
    assert!(r10 == 0x1010);
    assert!(r11 == 0x1111);
}
