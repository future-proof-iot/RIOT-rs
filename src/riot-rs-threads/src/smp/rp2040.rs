use crate::{
    arch::{Arch, Cpu},
    THREADS,
};

use super::Multicore;
use critical_section::CriticalSection;
use embassy_rp::{
    multicore::{spawn_core1, Stack},
    peripherals::CORE1,
};
use rp_pac::SIO;

pub struct Chip;

impl Multicore for Chip {
    const CORES: u32 = 2;

    fn cpuid() -> u32 {
        SIO.cpuid().read()
    }

    fn startup_cores() {
        let stack: &'static mut Stack<4096> = static_cell::make_static!(Stack::new());
        let start_threading = move || {
            let cpuid = Self::cpuid();
            let cs = unsafe { CriticalSection::new() };
            let next_sp = THREADS.with_mut_cs(cs, |mut threads| {
                // FIXME: this is a hack!
                let next_pid = threads.runqueue.get_next().unwrap() + (cpuid as u8);
                threads.current_threads[cpuid as usize] = Some(next_pid);
                threads.threads[next_pid as usize].sp
            });
            Cpu::start_threading(next_sp);
            loop {}
        };
        unsafe {
            spawn_core1(CORE1::steal(), stack, start_threading);
        }
    }
}
