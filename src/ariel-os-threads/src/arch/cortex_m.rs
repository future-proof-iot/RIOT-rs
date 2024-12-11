use crate::{cleanup, Arch, Thread, SCHEDULER};
use core::arch::asm;
use core::ptr::write_volatile;
use cortex_m::peripheral::{scb::SystemHandler, SCB};

#[cfg(not(any(armv6m, armv7m, armv8m)))]
compile_error!("no supported ARM variant selected");

pub struct Cpu;

impl Arch for Cpu {
    /// Callee-save registers.
    type ThreadData = [usize; 8];

    const DEFAULT_THREAD_DATA: Self::ThreadData = [0; 8];

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

        thread.sp = stack_pos as usize;
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
#[no_mangle]
#[allow(non_snake_case)]
unsafe extern "C" fn PendSV() {
    unsafe {
        asm!(
            "
            bl {sched}
            cmp r0, #0
            /* label rules:
             * - number only
             * - no combination of *only* [01]
             * - add f or b for 'next matching forward/backward'
             * so let's use '99' forward ('99f')
             */
            beq 99f

            msr.n psp, r0

            // r1 == 0 means that there was no previous thread.
            // This is only the case if the scheduler was triggered for the first time,
            // which also means that next thread has no stored context yet.
            // Storing and loading of r4-r11 therefore can be skipped.
            cmp r1, #0
            beq 99f

            stmia r1, {{r4-r11}}
            ldmia r2, {{r4-r11}}
            99:
            movw LR, #0xFFFd
            movt LR, #0xFFFF
            bx LR
            ",
            sched = sym sched,
            options(noreturn)
        )
    };
}

#[cfg(any(armv6m))]
#[naked]
#[no_mangle]
#[allow(non_snake_case)]
unsafe extern "C" fn PendSV() {
    unsafe {
        asm!(
            "
            bl {sched}
            cmp r0, #0
            beq 99f

            msr.n psp, r0

            // r1 == 0 means that there was no previous thread.
            // This is only the case if the scheduler was triggered for the first time,
            // which also means that next thread has no stored context yet.
            // Storing and loading of r4-r11 therefore can be skipped.
            cmp r1, #0
            beq 99f

            //stmia r1!, {{r4-r7}}
            str r4, [r1, #16]
            str r5, [r1, #20]
            str r6, [r1, #24]
            str r7, [r1, #28]

            mov  r4, r8
            mov  r5, r9
            mov  r6, r10
            mov  r7, r11

            str r4, [r1, #0]
            str r5, [r1, #4]
            str r6, [r1, #8]
            str r7, [r1, #12]

            //
            ldmia r2!, {{r4-r7}}
            mov r11, r7
            mov r10, r6
            mov r9,  r5
            mov r8,  r4
            ldmia r2!, {{r4-r7}}

            99:
            ldr r0, 999f
            mov LR, r0
            bx lr

            .align 4
            999:
            .word 0xFFFFFFFD
            ",
            sched = sym sched,
            options(noreturn)
        )
    };
}

/// Schedule the next thread.
///
/// It selects the next thread that should run from the runqueue.
/// This may be current thread, or a new one.
///
/// Returns:
/// - `0` in `r0` if the next thread in the runqueue is the currently running thread
/// - Else it writes into the following registers:
///   - `r1`: pointer to [`Thread::high_regs`] from old thread (to store old register state)
///           or null pointer if there was no previously running thread.
///   - `r2`: pointer to [`Thread::high_regs`] from new thread (to load new register state)
///   - `r0`: stack-pointer for new thread
///
/// This function is called in PendSV.
unsafe fn sched() -> (u32, u64) {
    let (next_sp, current_high_regs, next_high_regs) = loop {
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
            if let Some(ref mut current_pid_ref) = scheduler.current_pid_mut() {
                if next_pid == *current_pid_ref {
                    return Some((0, 0, 0));
                }
                let current_pid = *current_pid_ref;
                *current_pid_ref = next_pid;
                let current = scheduler.get_unchecked_mut(current_pid);
                current.sp = cortex_m::register::psp::read() as usize;
                current_high_regs = current.data.as_ptr();
            } else {
                *scheduler.current_pid_mut() = Some(next_pid);
            }

            let next = scheduler.get_unchecked(next_pid);
            let next_high_regs = next.data.as_ptr();
            let next_sp = next.sp;

            Some((
                next_sp as u32,
                current_high_regs as u32,
                next_high_regs as u32,
            ))
        }) {
            break res;
        }
    };

    // The caller (`PendSV`) expects these three pointers in r0, r1 and r2:
    // r0 = &next.sp
    // r1 = &current.high_regs
    // r2 = &next.high_regs
    (
        next_sp,
        ((current_high_regs as u64) | (next_high_regs as u64) << 32),
    )
}
