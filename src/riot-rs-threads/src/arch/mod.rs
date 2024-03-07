/// Arch-specific implementations for the scheduler.
pub trait Arch {
    const DEFAULT_THREAD_DATA: Self::ThreadData;

    type ThreadData;

    /// Sets up the stack for newly created threads and returns the sp.
    ///
    /// After running this, the stack should look as if the thread was
    /// interrupted by an ISR.
    ///
    /// It sets up the stack so when the context is switched to this thread,
    /// it starts executing `func` with argument `arg`.
    /// Furthermore, it sets up the link-register with the [`crate::cleanup`] function that
    /// will be executed after the thread function returned.
    fn setup_stack(stack: &mut [u8], func: usize, arg: usize) -> usize;

    /// Trigger a context switch.
    fn schedule();

    /// Setup and initiate the first context switch.
    fn start_threading(next_sp: usize);
}

cfg_if::cfg_if! {
    if #[cfg(context = "cortex-m")] {
        mod cortex_m;
        pub use cortex_m::Cpu;
    }
    else {
        pub struct Cpu;
        impl Arch for Cpu {
            type ThreadData = ();
            const DEFAULT_THREAD_DATA: Self::ThreadData = ();

            fn setup_stack(_: &mut [u8], _: usize, _: usize) -> usize {
                unimplemented!()
            }
            fn start_threading(_: usize) {
                unimplemented!()
            }
            fn schedule() {
                unimplemented!()
            }
        }
    }
}

pub type ThreadData = <Cpu as Arch>::ThreadData;

pub fn schedule() {
    Cpu::schedule()
}
