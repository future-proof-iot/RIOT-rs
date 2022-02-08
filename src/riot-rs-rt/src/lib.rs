#![no_std]
#![cfg_attr(test, no_main)]
//
#![allow(incomplete_features)]
// - const_generics

// features
#![feature(naked_functions)]
#![feature(fn_traits)]
#![feature(in_band_lifetimes)]
// clist / memoffset
#![feature(const_ptr_offset_from)]
// for msg_content_t union
// error[E0658]: unions with non-`Copy` fields other than `ManuallyDrop<T>` are unstable
#![feature(untagged_unions)]
// testing
#![feature(custom_test_frameworks)]
#![test_runner(crate::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]
pub mod testing;

use cortex_m as _;
use cortex_m_rt::{entry, exception, ExceptionFrame, __RESET_VECTOR};

use core::panic::PanicInfo;

pub mod debug {
    pub use cortex_m_semihosting::debug::{exit, EXIT_FAILURE, EXIT_SUCCESS};
    pub use cortex_m_semihosting::hprint as print;
    pub use cortex_m_semihosting::hprintln as println;
}

// Table 2.5
// http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dui0553a/CHDBIBGJ.html
pub fn ipsr_isr_number_to_str(isr_number: usize) -> &'static str {
    match isr_number {
        0 => "Thread Mode",
        1 => "Reserved",
        2 => "NMI",
        3 => "HardFault",
        4 => "MemManage",
        5 => "BusFault",
        6 => "UsageFault",
        7..=10 => "Reserved",
        11 => "SVCall",
        12 => "Reserved for Debug",
        13 => "Reserved",
        14 => "PendSV",
        15 => "SysTick",
        16..=255 => "IRQn",
        _ => "(Unknown! Illegal value?)",
    }
}

/// Extra verbose Cortex-M HardFault handler
///
/// (copied from Tock OS)
#[allow(non_snake_case)]
#[exception]
unsafe fn HardFault(ef: &ExceptionFrame) -> ! {
    use core::arch::asm;
    asm!("bkpt");

    let mode_str = "Kernel";

    let shcsr: u32 = core::ptr::read_volatile(0xE000ED24 as *const u32);
    let cfsr: u32 = core::ptr::read_volatile(0xE000ED28 as *const u32);
    let hfsr: u32 = core::ptr::read_volatile(0xE000ED2C as *const u32);
    let mmfar: u32 = core::ptr::read_volatile(0xE000ED34 as *const u32);
    let bfar: u32 = core::ptr::read_volatile(0xE000ED38 as *const u32);

    let iaccviol = (cfsr & 0x01) == 0x01;
    let daccviol = (cfsr & 0x02) == 0x02;
    let munstkerr = (cfsr & 0x08) == 0x08;
    let mstkerr = (cfsr & 0x10) == 0x10;
    let mlsperr = (cfsr & 0x20) == 0x20;
    let mmfarvalid = (cfsr & 0x80) == 0x80;

    let ibuserr = ((cfsr >> 8) & 0x01) == 0x01;
    let preciserr = ((cfsr >> 8) & 0x02) == 0x02;
    let impreciserr = ((cfsr >> 8) & 0x04) == 0x04;
    let unstkerr = ((cfsr >> 8) & 0x08) == 0x08;
    let stkerr = ((cfsr >> 8) & 0x10) == 0x10;
    let lsperr = ((cfsr >> 8) & 0x20) == 0x20;
    let bfarvalid = ((cfsr >> 8) & 0x80) == 0x80;

    let undefinstr = ((cfsr >> 16) & 0x01) == 0x01;
    let invstate = ((cfsr >> 16) & 0x02) == 0x02;
    let invpc = ((cfsr >> 16) & 0x04) == 0x04;
    let nocp = ((cfsr >> 16) & 0x08) == 0x08;
    let unaligned = ((cfsr >> 16) & 0x100) == 0x100;
    let divbysero = ((cfsr >> 16) & 0x200) == 0x200;

    let vecttbl = (hfsr & 0x02) == 0x02;
    let forced = (hfsr & 0x40000000) == 0x40000000;

    let xpsr = ef.xpsr();

    let ici_it = (((xpsr >> 25) & 0x3) << 6) | ((xpsr >> 10) & 0x3f);
    let thumb_bit = ((xpsr >> 24) & 0x1) == 1;
    let exception_number = (xpsr & 0x1ff) as usize;

    panic!(
        "{} HardFault.\r\n\
         \tKernel version {}\r\n\
         \tr0  0x{:x}\r\n\
         \tr1  0x{:x}\r\n\
         \tr2  0x{:x}\r\n\
         \tr3  0x{:x}\r\n\
         \tr12 0x{:x}\r\n\
         \tlr  0x{:x}\r\n\
         \tpc  0x{:x}\r\n\
         \tprs 0x{:x} [ N {} Z {} C {} V {} Q {} GE {}{}{}{} ; ICI.IT {} T {} ; Exc {}-{} ]\r\n\
         \tsp  0x{:x}\r\n\
         \ttop of stack     0x{:x}\r\n\
         \tbottom of stack  0x{:x}\r\n\
         \tSHCSR 0x{:x}\r\n\
         \tCFSR  0x{:x}\r\n\
         \tHSFR  0x{:x}\r\n\
         \tInstruction Access Violation:       {}\r\n\
         \tData Access Violation:              {}\r\n\
         \tMemory Management Unstacking Fault: {}\r\n\
         \tMemory Management Stacking Fault:   {}\r\n\
         \tMemory Management Lazy FP Fault:    {}\r\n\
         \tInstruction Bus Error:              {}\r\n\
         \tPrecise Data Bus Error:             {}\r\n\
         \tImprecise Data Bus Error:           {}\r\n\
         \tBus Unstacking Fault:               {}\r\n\
         \tBus Stacking Fault:                 {}\r\n\
         \tBus Lazy FP Fault:                  {}\r\n\
         \tUndefined Instruction Usage Fault:  {}\r\n\
         \tInvalid State Usage Fault:          {}\r\n\
         \tInvalid PC Load Usage Fault:        {}\r\n\
         \tNo Coprocessor Usage Fault:         {}\r\n\
         \tUnaligned Access Usage Fault:       {}\r\n\
         \tDivide By Zero:                     {}\r\n\
         \tBus Fault on Vector Table Read:     {}\r\n\
         \tForced Hard Fault:                  {}\r\n\
         \tFaulting Memory Address: (valid: {}) {:#010X}\r\n\
         \tBus Fault Address:       (valid: {}) {:#010X}\r\n\
         ",
        mode_str,
        option_env!("RIOTCORE_KERNEL_VERSION").unwrap_or("unknown"),
        ef.r0(),
        ef.r1(),
        ef.r2(),
        ef.r3(),
        ef.r12(),
        ef.lr(),
        ef.pc(),
        xpsr,
        (xpsr >> 31) & 0x1,
        (xpsr >> 30) & 0x1,
        (xpsr >> 29) & 0x1,
        (xpsr >> 28) & 0x1,
        (xpsr >> 27) & 0x1,
        (xpsr >> 19) & 0x1,
        (xpsr >> 18) & 0x1,
        (xpsr >> 17) & 0x1,
        (xpsr >> 16) & 0x1,
        ici_it,
        thumb_bit,
        exception_number,
        ipsr_isr_number_to_str(exception_number),
        0u32,
        0u32,
        0u32,
        // faulting_stack as u32,
        // (_estack as *const ()) as u32,
        // (&_sstack as *const u32) as u32,
        shcsr,
        cfsr,
        hfsr,
        iaccviol,
        daccviol,
        munstkerr,
        mstkerr,
        mlsperr,
        ibuserr,
        preciserr,
        impreciserr,
        unstkerr,
        stkerr,
        lsperr,
        undefinstr,
        invstate,
        invpc,
        nocp,
        unaligned,
        divbysero,
        vecttbl,
        forced,
        mmfarvalid,
        mmfar,
        bfarvalid,
        bfar
    );
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    #[cfg(not(feature = "silent-panic"))]
    {
        debug::println!("panic: {}\n", _info);
        debug::exit(debug::EXIT_FAILURE);
    }
    loop {}
}

#[exception]
unsafe fn DefaultHandler(_irqn: i16) {
    #[cfg(not(feature = "silent-panic"))]
    {
        debug::println!("IRQn = {}", _irqn);
        debug::exit(debug::EXIT_FAILURE);
    }
    loop {}
}

extern "C" {
    fn riot_rs_rt_startup();
}

#[entry]
fn main() -> ! {
    // First, configure vector table address.
    // This is necessary when the vector table is not at its default position,
    // e.g., when there's a bootloader the default address.
    // Here, we're deriving the vector table address from the reset vector,
    // which is always the second entry in the vector table, after the initial
    // ISR stack pointer.
    // TODO: make cortex_m only
    unsafe {
        (*cortex_m::peripheral::SCB::PTR)
            .vtor
            .write(&__RESET_VECTOR as *const _ as u32 - 4)
    };

    debug::println!("riot_rs_rt::main()");

    #[cfg(not(test))]
    unsafe {
        riot_rs_rt_startup();
    }

    #[cfg(test)]
    test_main();
    loop {}
}

#[test_case]
fn test_trivial() {
    assert!(1 == 1);
}
