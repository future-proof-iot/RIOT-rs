use crate::arch::{Arch as _, Cpu};

use embassy_rp::{
    multicore::{spawn_core1, Stack},
    peripherals::CORE1,
};
use rp_pac::SIO;
use static_cell::ConstStaticCell;

use super::{CoreId, Multicore};

pub struct Chip;

impl Multicore for Chip {
    const CORES: u32 = 2;

    fn core_id() -> CoreId {
        CoreId(SIO.cpuid().read() as u8)
    }

    fn startup_other_cores() {
        // TODO: How much stack do we really need here?
        static STACK: ConstStaticCell<Stack<4096>> = ConstStaticCell::new(Stack::new());
        // Trigger scheduler.
        let start_threading = move || {
            Cpu::start_threading();
            unreachable!()
        };
        unsafe {
            spawn_core1(CORE1::steal(), STACK.take(), start_threading);
        }
    }

    fn sev() {
        cortex_m::asm::sev()
    }
}
