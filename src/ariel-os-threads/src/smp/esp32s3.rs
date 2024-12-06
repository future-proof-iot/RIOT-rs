use esp_hal::{
    cpu_control::{CpuControl, Stack},
    interrupt,
    peripherals::{Interrupt, CPU_CTRL, SYSTEM},
    Cpu,
};

use static_cell::ConstStaticCell;

use super::{CoreId, Multicore, ISR_STACKSIZE_CORE1};

impl From<Cpu> for CoreId {
    fn from(value: Cpu) -> Self {
        match value {
            Cpu::ProCpu => CoreId(0),
            Cpu::AppCpu => CoreId(1),
        }
    }
}

pub struct Chip;

impl Multicore for Chip {
    const CORES: u32 = 2;
    const IDLE_THREAD_STACK_SIZE: usize = 2048;

    fn core_id() -> CoreId {
        esp_hal::Cpu::current().into()
    }

    fn startup_other_cores() {
        // TODO: How much stack do we really need here?
        static STACK: ConstStaticCell<Stack<ISR_STACKSIZE_CORE1>> =
            ConstStaticCell::new(Stack::new());
        // Trigger scheduler.
        let start_threading = move || {
            // Use `CPU_INTR1` to trigger the scheduler on our second core.
            // We need to use a different interrupt here than on the first core so that
            // we specifically trigger the scheduler on one or the other core.
            interrupt::disable(esp_hal::Cpu::ProCpu, Interrupt::FROM_CPU_INTR1);
            Self::schedule_on_core(Self::core_id());
            // Panics if `FROM_CPU_INTR1` is among `esp_hal::interrupt::RESERVED_INTERRUPTS`,
            // which isn't the case.
            interrupt::enable(Interrupt::FROM_CPU_INTR1, interrupt::Priority::min()).unwrap();

            unreachable!()
        };

        let mut cpu_ctrl = unsafe { CpuControl::new(CPU_CTRL::steal()) };
        let guard = cpu_ctrl.start_app_core(STACK.take(), start_threading);

        // Dropping the guard would park the other core.
        core::mem::forget(guard)
    }

    fn schedule_on_core(id: CoreId) {
        let ptr = unsafe { &*SYSTEM::PTR };
        let mut id = id.0;
        let already_set = match id {
            0 => ptr
                .cpu_intr_from_cpu_0()
                .read()
                .cpu_intr_from_cpu_0()
                .bit_is_set(),
            1 => ptr
                .cpu_intr_from_cpu_1()
                .read()
                .cpu_intr_from_cpu_1()
                .bit_is_set(),
            _ => unreachable!(),
        };
        if already_set {
            // If a scheduling attempt is already pending, there must have been multiple
            // changes in the runqueue at the same time.
            // Trigger the scheduler on the other core as well to make sure that both schedulers
            // have the most recent runqueue state.
            id ^= 1;
        }
        match id {
            0 => ptr
                .cpu_intr_from_cpu_0()
                .write(|w| w.cpu_intr_from_cpu_0().set_bit()),
            1 => ptr
                .cpu_intr_from_cpu_1()
                .write(|w| w.cpu_intr_from_cpu_1().set_bit()),
            _ => unreachable!(),
        };
    }
}
