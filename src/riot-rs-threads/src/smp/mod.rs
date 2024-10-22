/// ID of a physical core.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CoreId(pub(crate) u8);

impl CoreId {
    /// Creates a new [`CoreId`].
    ///
    /// # Panics
    ///
    /// Panics if `value` >= [`Chip::CORES`].
    pub fn new(value: u8) -> Self {
        if value >= Chip::CORES as u8 {
            panic!(
                "Invalid CoreId {}: only core ids 0..{} available.",
                value,
                Chip::CORES
            )
        }
        Self(value)
    }
}

impl From<CoreId> for usize {
    fn from(value: CoreId) -> Self {
        value.0 as usize
    }
}

pub trait Multicore {
    /// Number of available core.
    const CORES: u32;
    /// Stack size for the idle threads.
    const IDLE_THREAD_STACK_SIZE: usize = 256;

    /// Returns the ID of the current core.
    fn core_id() -> CoreId;

    /// Starts other available cores.
    ///
    /// This is called at boot time by the first core.
    fn startup_other_cores();

    fn sev();

    /// Triggers the scheduler on core `id`.
    fn schedule_on_core(id: CoreId);
}

cfg_if::cfg_if! {
    if #[cfg(context = "rp2040")] {
        mod rp2040;
        pub use rp2040::Chip;
    }
    else {
        use crate::{Arch as _, Cpu};

        pub struct Chip;
        impl Multicore for Chip {
            const CORES: u32 = 1;

            fn core_id() -> CoreId {
                CoreId(0)
            }

            fn startup_other_cores() {}

            fn sev() {}

            fn schedule_on_core(_id: CoreId) {
                Cpu::schedule();
            }
        }
    }
}

pub fn sev() {
    Chip::sev()
}

/// Triggers the scheduler on core `id`.
pub fn schedule_on_core(id: CoreId) {
    Chip::schedule_on_core(id)
}
