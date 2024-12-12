use crate::{cleanup, Arch, Thread, SCHEDULER};
use core::{
    arch::{asm, naked_asm},
    ptr::write_volatile,
};
use cortex_m::peripheral::{scb::SystemHandler, SCB};

#[cfg(not(any(armv6m, armv7m, armv8m)))]
compile_error!("no supported ARM variant selected");

pub struct Cpu;

#[derive(Default, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ThreadData {
    sp: usize,
    high_regs: [usize; 8],
}

impl Arch for Cpu {
    /// Callee-save registers.
    type ThreadData = ThreadData;

    const DEFAULT_THREAD_DATA: Self::ThreadData = ThreadData {
        sp: 0,
        high_regs: [0; 8],
    };

    /// The exact order in which Cortex-M pushes the registers to the stack when
    /// entering the ISR is:
    ///
    /// +---------+ <- sp
    /// |   r0    |
    /// |   r1    |
    /// |   r2    |
    /// |   r3    |
    /// |   r12   |
    /// |   LR    |
    /// |   PC    |
    /// |   PSR   |
    /// +---------+
    fn setup_stack(thread: &mut Thread, stack: &mut [u8], func: usize, arg: usize) {
        let stack_start = stack.as_ptr() as usize;

        // 1. The stack starts at the highest address and grows downwards.
        // 2. A full stored context also contains R4-R11 and the stack pointer,
        //    thus an additional 36 bytes need to be reserved.
        // 3. Cortex-M expects the SP to be 8 byte aligned, so we chop the lowest
        //    7 bits by doing `& 0xFFFFFFF8`.
        let stack_pos = ((stack_start + stack.len() - 36) & 0xFFFFFFF8) as *mut usize;

        unsafe {
            write_volatile(stack_pos.offset(0), arg); // -> R0
            write_volatile(stack_pos.offset(1), 1); // -> R1
            write_volatile(stack_pos.offset(2), 2); // -> R2
            write_volatile(stack_pos.offset(3), 3); // -> R3
            write_volatile(stack_pos.offset(4), 12); // -> R12
            write_volatile(stack_pos.offset(5), cleanup as usize); // -> LR
            write_volatile(stack_pos.offset(6), func); // -> PC
            write_volatile(stack_pos.offset(7), 0x01000000); // -> APSR
        }

        thread.data.sp = stack_pos as usize;
    }

    /// Triggers a PendSV exception.
    #[inline(always)]
    fn schedule() {
        SCB::set_pendsv();
        cortex_m::asm::isb();
    }

    #[inline(always)]
    fn start_threading() {
        unsafe {
            // Make sure PendSV has a low priority.
            let mut p = cortex_m::Peripherals::steal();
            p.SCB.set_priority(SystemHandler::PendSV, 0xFF);
        }
        Self::schedule();
    }

    fn wfi() {
        cortex_m::asm::wfi();

        // see https://cliffle.com/blog/stm32-wfi-bug/
        #[cfg(context = "stm32")]
        cortex_m::asm::isb();
    }
}

#[cfg(any(armv7m, armv8m))]
#[naked]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
unsafe extern "C" fn PendSV() {
    unsafe {
        naked_asm!(
            "
            bl {sched}

            // r0 == 0 means that
            // a) there was no previous thread, or
            // This is only the case if the scheduler was triggered for the first time,
            // which also means that next thread has no stored context yet.
            // b) the current thread didn't change.
            //
            // In both cases, storing and loading of r4-r11 can be skipped.
            cmp r0, #0

            /* label rules:
             * - number only
             * - no combination of *only* [01]
             * - add f or b for 'next matching forward/backward'
             */
            beq 99f

            stmia r0, {{r4-r11}}
            ldmia r1, {{r4-r11}}

            99:
            movw LR, #0xFFFd
            movt LR, #0xFFFF
            bx LR
            ",
            sched = sym sched,
        )
    };
}

#[cfg(any(armv6m))]
#[naked]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
unsafe extern "C" fn PendSV() {
    unsafe {
        naked_asm!(
            "
            bl {sched}

            // r0 == 0 means that
            // a) there was no previous thread, or
            // This is only the case if the scheduler was triggered for the first time,
            // which also means that next thread has no stored context yet.
            // b) the current thread didn't change.
            //
            // In both cases, storing and loading of r4-r11 can be skipped.
            cmp r0, #0

            //stmia r1!, {{r4-r7}}
            str r4, [r0, #16]
            str r5, [r0, #20]
            str r6, [r0, #24]
            str r7, [r0, #28]

            mov  r4, r8
            mov  r5, r9
            mov  r6, r10
            mov  r7, r11

            str r4, [r0, #0]
            str r5, [r0, #4]
            str r6, [r0, #8]
            str r7, [r0, #12]

            //
            ldmia r1!, {{r4-r7}}
            mov r11, r7
            mov r10, r6
            mov r9,  r5
            mov r8,  r4
            ldmia r1!, {{r4-r7}}

            99:
            ldr r0, 999f
            mov LR, r0
            bx lr

            .align 4
            999:
            .word 0xFFFFFFFD
            ",
            sched = sym sched,
        )
    };
}

/// Schedule the next thread.
///
/// It selects the next thread that should run from the runqueue.
/// This may be current thread, or a new one.
///
/// Returns:
///   - `r0`: pointer to [`Thread::high_regs`] from old thread (to store old register state)
///           or null pointer if there was no previously running thread, or the currently running
///           thread should not be changed.
///   - `r1`: pointer to [`Thread::high_regs`] from new thread (to load new register state)
///
/// This function is called in PendSV from assembly, so it must be `extern "C"`.
unsafe extern "C" fn sched() -> u64 {
    let (current_high_regs, next_high_regs) = loop {
        if let Some(res) = critical_section::with(|cs| {
            let scheduler = unsafe { &mut *SCHEDULER.as_ptr(cs) };

            #[cfg(feature = "multi-core")]
            scheduler.add_current_thread_to_rq();

            let next_pid = match scheduler.get_next_pid() {
                Some(pid) => pid,
                None => {
                    #[cfg(feature = "multi-core")]
                    unreachable!("At least one idle thread is always present for each core.");

                    #[cfg(not(feature = "multi-core"))]
                    {
                        Cpu::wfi();
                        // this fence seems necessary, see #310.
                        core::sync::atomic::fence(core::sync::atomic::Ordering::Acquire);
                        return None;
                    }
                }
            };

            // `current_high_regs` will be null if there is no current thread.
            // This is only the case once, when the very first thread starts running.
            // The returned `r1` therefore will be null, and saving/ restoring
            // the context is skipped.
            let mut current_high_regs = core::ptr::null();
            if let Some(current_pid_ref) = scheduler.current_pid_mut() {
                if next_pid == *current_pid_ref {
                    return Some((0, 0));
                }
                let current_pid = *current_pid_ref;
                *current_pid_ref = next_pid;
                let current = scheduler.get_unchecked_mut(current_pid);
                current.data.sp = cortex_m::register::psp::read() as usize;
                current_high_regs = current.data.high_regs.as_ptr();
            } else {
                *scheduler.current_pid_mut() = Some(next_pid);
            }

            let next = scheduler.get_unchecked(next_pid);
            // SAFETY: changing the PSP as part of context switch
            unsafe { cortex_m::register::psp::write(next.data.sp as u32) };
            let next_high_regs = next.data.high_regs.as_ptr();

            Some((current_high_regs as u32, next_high_regs as u32))
        }) {
            break res;
        }
    };

    // The caller (`PendSV`) expects these two pointers in r0 and r1:
    // r0 = &current.data.high_regs (or 0)
    // r1 = &next.data.high_regs
    // The C ABI on ARM (AAPCS) defines u64 to be returned in r0 and r1, so we use that to fit our
    // values in there. `extern "C"` on this function ensures the Rust compiler adheres to those
    // rules.
    // See https://github.com/ARM-software/abi-aa/blob/a82eef0433556b30539c0d4463768d9feb8cfd0b/aapcs32/aapcs32.rst#6111handling-values-larger-than-32-bits
    (current_high_regs as u64) | (next_high_regs as u64) << 32
}
