use crate::{cleanup, Arch, Thread, THREADS};
#[cfg(context = "esp32c6")]
use esp_hal::peripherals::INTPRI as SYSTEM;
#[cfg(context = "esp32c3")]
use esp_hal::peripherals::SYSTEM;
use esp_hal::{
    interrupt::{self, TrapFrame},
    peripherals::Interrupt,
    prelude::*,
    riscv, Cpu as EspHalCpu,
};

pub struct Cpu;

impl Arch for Cpu {
    type ThreadData = TrapFrame;
    const DEFAULT_THREAD_DATA: Self::ThreadData = default_trap_frame();

    /// Triggers software interrupt for the context switch.
    fn schedule() {
        unsafe {
            (&*SYSTEM::PTR)
                .cpu_intr_from_cpu_1()
                .modify(|_, w| w.cpu_intr_from_cpu_1().set_bit());
        }
    }

    /// On RISC-V (ESP32), the stack doesn't need to be set up with any register values since
    /// they are restored from the stored [`TrapFrame`].
    fn setup_stack(thread: &mut Thread, stack: &mut [u8], func: usize, arg: usize) {
        let stack_start = stack.as_ptr() as usize;
        // 16 byte alignment.
        let stack_pos = (stack_start + stack.len()) & 0xFFFFFFE0;
        // Set up PC, SP, RA and first argument for function.
        thread.sp = stack_pos;
        thread.data.sp = stack_pos;
        thread.data.a0 = arg;
        thread.data.ra = cleanup as usize;
        thread.data.pc = func;
    }

    /// Enable and trigger the appropriate software interrupt.
    fn start_threading() {
        interrupt::disable(EspHalCpu::ProCpu, Interrupt::FROM_CPU_INTR1);
        Self::schedule();
        // TODO: handle unwrap error?
        interrupt::enable(Interrupt::FROM_CPU_INTR1, interrupt::Priority::min()).unwrap();
    }
}

const fn default_trap_frame() -> TrapFrame {
    TrapFrame {
        ra: 0,
        t0: 0,
        t1: 0,
        t2: 0,
        t3: 0,
        t4: 0,
        t5: 0,
        t6: 0,
        a0: 0,
        a1: 0,
        a2: 0,
        a3: 0,
        a4: 0,
        a5: 0,
        a6: 0,
        a7: 0,
        s0: 0,
        s1: 0,
        s2: 0,
        s3: 0,
        s4: 0,
        s5: 0,
        s6: 0,
        s7: 0,
        s8: 0,
        s9: 0,
        s10: 0,
        s11: 0,
        gp: 0,
        tp: 0,
        sp: 0,
        pc: 0,
        mstatus: 0,
        mcause: 0,
        mtval: 0,
    }
}

/// Copies the register state from `src` to `dst`.
///
/// It copies state from the `TrapFrame` except for CSR registers
/// `mstatus`, `mcause` and `mtval`.
fn copy_registers(src: &TrapFrame, dst: &mut TrapFrame) {
    let (mstatus, mcause, mtval) = (dst.mstatus, dst.mcause, dst.mtval);
    dst.clone_from(src);
    dst.mstatus = mstatus;
    dst.mcause = mcause;
    dst.mtval = mtval;
}

/// Handler for software interrupt 0, which we use for context switching.
#[allow(non_snake_case)]
#[interrupt]
fn FROM_CPU_INTR1(trap_frame: &mut TrapFrame) {
    unsafe {
        // clear FROM_CPU_INTR1
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
            let next_pid = match threads.runqueue.get_next() {
                Some(pid) => pid,
                None => {
                    riscv::asm::wfi();
                    return false;
                }
            };

            if let Some(current_pid) = threads.current_pid() {
                if next_pid == current_pid {
                    return true;
                }
                copy_registers(trap_frame, &mut threads.threads[current_pid as usize].data);
            }
            threads.current_thread = Some(next_pid);
            copy_registers(&threads.threads[next_pid as usize].data, trap_frame);
            true
        }) {
            break;
        }
    }
}
