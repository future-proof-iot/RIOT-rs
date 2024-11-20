//! Provides an executor that should run inside a thread.
#![deny(missing_docs)]

// This is based on the upstream embassy cortex-m interrupt executor.

use core::marker::PhantomData;

use ariel_os_threads::{current_pid, thread_flags, thread_flags::ThreadFlags, ThreadId};
use embassy_executor::{raw, Spawner};

// This is only used between `__pender` and `Executor::run( )`, actual flag
// doesn't matter.
const THREAD_FLAG_WAKEUP: ThreadFlags = 0x01;

#[export_name = "__pender"]
fn __pender(context: *mut ()) {
    // SAFETY: `context` is a `ThreadId` passed by `ThreadExecutor::new`.
    let thread_id = ThreadId::new(context as usize as u8);

    thread_flags::set(thread_id, THREAD_FLAG_WAKEUP);
}

/// Thread mode executor for Ariel OS threads.
pub struct Executor {
    inner: raw::Executor,
    // This executor is tied to a specific thread by storing the `ThreadId` inside
    // the inner `raw::Executor`. It thus cannot be `Send`.
    not_send: PhantomData<*mut ()>,
}

impl Executor {
    /// Creates a new [`Executor`].
    ///
    /// This must be called from the thread that will actually poll the executor.
    /// Otherwise, the internally used thread flag will be sent to the wrong thread,
    /// causing the executor thread to never wake up.
    ///
    /// # Panics
    ///
    /// This function panics when called without a running thread.
    pub fn new() -> Self {
        let current_thread = current_pid().unwrap();
        Self {
            inner: raw::Executor::new(usize::from(current_thread) as *mut ()),
            not_send: PhantomData,
        }
    }

    /// Runs the executor.
    ///
    /// The `init` closure is called with a [`Spawner`] that spawns tasks on
    /// this executor. Use it to spawn the initial task(s). After `init` returns,
    /// the executor starts running the tasks.
    ///
    /// To spawn more tasks later, you may keep copies of the [`Spawner`] (it is `Copy`),
    /// for example by passing it as an argument to the initial tasks.
    ///
    /// This function requires `&'static mut self`. This means you have to store the
    /// [`Executor`] instance in a place where it'll live forever and grants you mutable
    /// access. There's a few ways to do this:
    ///
    /// - a [`StaticCell`](https://docs.rs/static_cell/latest/static_cell/) (safe)
    /// - a `static mut` (unsafe)
    /// - a local variable in a function you know never returns (like `fn main() -> !`), upgrading its lifetime with `transmute`. (unsafe)
    pub fn run(&'static mut self, init: impl FnOnce(Spawner)) -> ! {
        init(self.inner.spawner());

        loop {
            // SAFETY: `poll()` may net be called reentrantly on the same executor, which we don't.
            unsafe {
                self.inner.poll();
            };
            thread_flags::wait_any(THREAD_FLAG_WAKEUP);
        }
    }
}
