use crate::{cleanup, Arch, Thread, SCHEDULER};
#[cfg(context = "esp32c6")]
use esp_hal::peripherals::INTPRI as SYSTEM;
#[cfg(context = "esp32c3")]
use esp_hal::peripherals::SYSTEM;
use esp_hal::{
    interrupt::{self, TrapFrame},
    peripherals::Interrupt,
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
                .cpu_intr_from_cpu_0()
                .modify(|_, w| w.cpu_intr_from_cpu_0().set_bit());
        }
    }

    /// On RISC-V (ESP32), the stack doesn't need to be set up with any register values since
    /// they are restored from the stored [`TrapFrame`].
    fn setup_stack(thread: &mut Thread, stack: &mut [u8], func: usize, arg: usize) {
        let stack_start = stack.as_ptr() as usize;
        // 16 byte alignment.
        let stack_pos = (stack_start + stack.len()) & 0xFFFFFFE0;
        // Set up PC, SP, RA and first argument for function.
        thread.data.sp = stack_pos;
        thread.data.a0 = arg;
        thread.data.ra = cleanup as usize;
        thread.data.pc = func;
    }

    /// Enable and trigger the appropriate software interrupt.
    fn start_threading() {
        interrupt::disable(EspHalCpu::ProCpu, Interrupt::FROM_CPU_INTR0);
        Self::schedule();
        // Panics if `FROM_CPU_INTR0` is among `esp_hal::interrupt::RESERVED_INTERRUPTS`,
        // which isn't the case.
        interrupt::enable(Interrupt::FROM_CPU_INTR0, interrupt::Priority::min()).unwrap();
    }

    fn wfi() {
        riscv::asm::wfi();
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
#[unsafe(no_mangle)]
extern "C" fn FROM_CPU_INTR0(trap_frame: &mut TrapFrame) {
    unsafe {
        // clear FROM_CPU_INTR0
        (&*SYSTEM::PTR)
            .cpu_intr_from_cpu_0()
            .modify(|_, w| w.cpu_intr_from_cpu_0().clear_bit());

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
        if SCHEDULER.with_mut(|mut scheduler| {
            #[cfg(feature = "multi-core")]
            scheduler.add_current_thread_to_rq();

            let next_pid = match scheduler.get_next_pid() {
                Some(pid) => pid,
                None => {
                    Cpu::wfi();
                    return false;
                }
            };

            if let Some(current_pid) = scheduler.current_pid() {
                if next_pid == current_pid {
                    return true;
                }
                copy_registers(
                    trap_frame,
                    &mut scheduler.threads[usize::from(current_pid)].data,
                );
            }
            *scheduler.current_pid_mut() = Some(next_pid);

            copy_registers(&scheduler.get_unchecked(next_pid).data, trap_frame);
            true
        }) {
            break;
        }
    }
}
