use esp_hal::{
    interrupt,
    peripherals::{Interrupt, SYSTEM},
    trapframe::TrapFrame,
};

use crate::{cleanup, Arch, THREADS};

pub struct Cpu;

impl Arch for Cpu {
    type ThreadData = TrapFrame;
    const DEFAULT_THREAD_DATA: Self::ThreadData = default_trap_frame();

    fn schedule() {
        #[cfg(not(feature = "multicore"))]
        unsafe {
            (&*SYSTEM::PTR)
                .cpu_intr_from_cpu_0()
                .modify(|_, w| w.cpu_intr_from_cpu_0().set_bit());
        }
        #[cfg(feature = "multicore")]
        crate::smp::schedule_on_core(crate::core_id())
    }

    fn setup_stack(thread: &mut crate::thread::Thread, stack: &mut [u8], func: usize, arg: usize) {
        let stack_start = stack.as_ptr() as usize;
        let task_stack_ptr = stack_start + stack.len();
        // 16 byte alignment.
        let stack_pos = task_stack_ptr - (task_stack_ptr % 0x10);

        thread.sp = stack_pos;
        thread.data.A1 = stack_pos as u32;
        thread.data.A6 = arg as u32;
        // Usually A0 holds the return address.
        // However, xtensa features so-called Windowed registers, which allow
        // to shift the used registers when calling procedure.
        // The xtensa-lx-rt does this when calling the exception handler using
        // call4, which shifts the window by 4.
        // See `xtensa_lx_rt::exception::asm::__default_naked_exception`.
        thread.data.A4 = cleanup as u32;
        thread.data.PC = func as u32;

        // Copied from esp-wifi::preempt::preempt_xtensa

        // For windowed ABI set WOE and CALLINC (pretend task was 'call4'd).
        thread.data.PS = 0x00040000 | (1 & 3) << 16;
    }

    fn start_threading() {
        interrupt::disable(esp_hal::Cpu::ProCpu, Interrupt::FROM_CPU_INTR0);
        Self::schedule();
        // Panics if `FROM_CPU_INTR0` is among `esp_hal::interrupt::RESERVED_INTERRUPTS`,
        // which isn't the case.
        interrupt::enable(Interrupt::FROM_CPU_INTR0, interrupt::Priority::min()).unwrap();
    }

    fn wfi() {
        unsafe { core::arch::asm!("waiti 0") };
    }
}

const fn default_trap_frame() -> TrapFrame {
    TrapFrame::new()
}

/// Handler for software interrupt 0, which we use for context switching on core 0.
#[allow(non_snake_case)]
#[no_mangle]
extern "C" fn FROM_CPU_INTR0(trap_frame: &mut TrapFrame) {
    unsafe {
        // clear FROM_CPU_INTR0
        (&*SYSTEM::PTR)
            .cpu_intr_from_cpu_0()
            .modify(|_, w| w.cpu_intr_from_cpu_0().clear_bit());

        sched(trap_frame)
    }
}

#[cfg(feature = "multicore")]
/// Handler for software interrupt 1, which we use for context switching on core 1.
#[allow(non_snake_case)]
#[no_mangle]
extern "C" fn FROM_CPU_INTR1(trap_frame: &mut TrapFrame) {
    unsafe {
        // clear FROM_CPU_INTR0
        (&*SYSTEM::PTR)
            .cpu_intr_from_cpu_1()
            .modify(|_, w| w.cpu_intr_from_cpu_1().clear_bit());

        sched(trap_frame)
    }
}

/// Probes the runqueue for the next thread and switches context if needed.
///
/// # Safety
///
/// This method might switch the current register state that is saved in the
/// `trap_frame`.
/// It should only be called from inside the trap handler that is responsible for
/// context switching.
unsafe fn sched(trap_frame: &mut TrapFrame) {
    loop {
        if THREADS.with_mut(|mut threads| {
            #[cfg(feature = "multicore")]
            threads.add_current_thread_to_rq();

            let Some(next_pid) = threads.get_next_pid() else {
                return false;
            };

            if let Some(current_pid) = threads.current_pid() {
                if next_pid == current_pid {
                    return true;
                }
                threads.threads[usize::from(current_pid)].data = *trap_frame;
            }
            *threads.current_pid_mut() = Some(next_pid);

            *trap_frame = threads.threads[usize::from(next_pid)].data;
            true
        }) {
            break;
        }
        // The esp-hal implementation of critical-section doesn't disable all interrupts.
        // Thus we should release our hold on `THREADS` before we `waiti`, to prevent
        // that another interrupt handler will try to borrow it while we still have it borrowed.
        Cpu::wfi()
    }
}
