//! Multi-threading for RIOT-rs.
//!
//! Implements a scheduler based on fixed priorities and preemption.
//! Within one priority level, threads are scheduled cooperatively.
//! This means that there is no time slicing that would equally distribute CPU time among same-priority threads.
//! **Instead, you need to use [`yield_same()`] to explicitly yield to another thread with the same priority.**
//! If no thread is ready, the core is prompted to enter deep sleep until a next thread is ready.
//!
//! Threads should be implemented using the `riot_rs_macros::thread` proc macro, which takes care
//! of calling the necessary initialization methods and linking the thread function element it into the binary.
//! A [`ThreadId`] between 0 and [`THREADS_NUMOF`] is assigned to each thread in the order in
//! which the threads are declared.
//!
//! Optionally, the stacksize and a priority between 1 and [`SCHED_PRIO_LEVELS`] can be configured.
//! By default, the stack size is 2048 bytes and priority is 1.
//!
//! # Synchronization
//!
//! The `threading` module supports three basic synchronization primitives:
//! - [`Channel`](sync::Channel): synchronous (blocking) channel for sending data between threads
//! - [`Lock`](sync::Lock): basic locking object
//! - [`thread_flags`]: thread-flag implementation for signaling between threads

#![cfg_attr(not(test), no_std)]
#![feature(naked_functions)]
#![feature(used_with_arg)]
#![cfg_attr(target_arch = "xtensa", feature(asm_experimental_arch))]
// Disable indexing lints for now, possible panics are documented or rely on internally-enforced
// invariants
#![allow(clippy::indexing_slicing)]

mod arch;
mod autostart_thread;
mod ensure_once;
mod thread;
mod threadlist;

pub mod sync;
pub mod thread_flags;

#[doc(hidden)]
pub mod macro_reexports {
    // Used by `autostart_thread`
    pub use linkme;
    pub use paste;
    pub use static_cell;
}

pub use riot_rs_runqueue::{RunqueueId, ThreadId};
pub use thread_flags as flags;

use arch::{schedule, Arch, Cpu, ThreadData};
use ensure_once::EnsureOnce;
use riot_rs_runqueue::RunQueue;
use thread::{Thread, ThreadState};

/// The number of possible priority levels.
pub const SCHED_PRIO_LEVELS: usize = 12;

/// The maximum number of concurrent threads that can be created.
pub const THREADS_NUMOF: usize = 16;

static THREADS: EnsureOnce<Threads> = EnsureOnce::new(Threads::new());

pub type ThreadFn = fn();

#[linkme::distributed_slice]
pub static THREAD_FNS: [ThreadFn] = [..];

/// Struct holding all scheduler state
struct Threads {
    /// Global thread runqueue.
    runqueue: RunQueue<SCHED_PRIO_LEVELS, THREADS_NUMOF>,
    /// The actual TCBs.
    threads: [Thread; THREADS_NUMOF],
    /// `Some` when a thread is blocking another thread due to conflicting
    /// resource access.
    thread_blocklist: [Option<ThreadId>; THREADS_NUMOF],
    /// The currently running thread.
    current_thread: Option<ThreadId>,
}

impl Threads {
    const fn new() -> Self {
        Self {
            runqueue: RunQueue::new(),
            threads: [const { Thread::default() }; THREADS_NUMOF],
            thread_blocklist: [const { None }; THREADS_NUMOF],
            current_thread: None,
        }
    }

    // pub(crate) fn by_pid_unckecked(&mut self, thread_id: ThreadId) -> &mut Thread {
    //     &mut self.threads[thread_id as usize]
    // }

    /// Returns checked mutable access to the thread data of the currently
    /// running thread.
    ///
    /// Returns `None` if there is no current thread.
    fn current(&mut self) -> Option<&mut Thread> {
        self.current_thread
            .map(|tid| &mut self.threads[usize::from(tid)])
    }

    fn current_pid(&self) -> Option<ThreadId> {
        self.current_thread
    }

    /// Creates a new thread.
    ///
    /// This sets up the stack and TCB for this thread.
    ///
    /// Returns `None` if there is no free thread slot.
    fn create(
        &mut self,
        func: usize,
        arg: usize,
        stack: &'static mut [u8],
        prio: RunqueueId,
    ) -> Option<ThreadId> {
        let (thread, pid) = self.get_unused()?;
        Cpu::setup_stack(thread, stack, func, arg);
        thread.prio = prio;
        thread.pid = pid;
        thread.state = ThreadState::Paused;
        Some(pid)
    }

    /// Returns immutable access to any thread data.
    ///
    /// # Panics
    ///
    /// Panics if `thread_id` is >= [`THREADS_NUMOF`].
    /// If the thread for this `thread_id` is in an invalid state, the
    /// data in the returned [`Thread`] is undefined, i.e. empty or outdated.
    fn get_unchecked(&self, thread_id: ThreadId) -> &Thread {
        &self.threads[usize::from(thread_id)]
    }

    /// Returns mutable access to any thread data.
    ///
    /// # Panics
    ///
    /// Panics if `thread_id` is >= [`THREADS_NUMOF`].
    /// If the thread for this `thread_id` is in an invalid state, the
    /// data in the returned [`Thread`] is undefined, i.e. empty or outdated.
    fn get_unchecked_mut(&mut self, thread_id: ThreadId) -> &mut Thread {
        &mut self.threads[usize::from(thread_id)]
    }

    /// Returns an unused ThreadId / Thread slot.
    fn get_unused(&mut self) -> Option<(&mut Thread, ThreadId)> {
        for i in 0..THREADS_NUMOF {
            if self.threads[i].state == ThreadState::Invalid {
                return Some((&mut self.threads[i], ThreadId::new(i as u8)));
            }
        }
        None
    }

    /// Checks if a thread with valid state exists for this `thread_id`.
    fn is_valid_pid(&self, thread_id: ThreadId) -> bool {
        if usize::from(thread_id) >= THREADS_NUMOF {
            false
        } else {
            self.threads[usize::from(thread_id)].state != ThreadState::Invalid
        }
    }

    /// Sets the state of a thread and triggers the scheduler if needed.
    ///
    /// This function also handles adding/ removing the thread to the Runqueue depending
    /// on its previous or new state.
    ///
    /// # Panics
    ///
    /// Panics if `pid` is >= [`THREADS_NUMOF`].
    fn set_state(&mut self, pid: ThreadId, state: ThreadState) -> ThreadState {
        let thread = self.get_unchecked_mut(pid);
        let old_state = core::mem::replace(&mut thread.state, state);
        let prio = thread.prio;

        if state == ThreadState::Running {
            self.runqueue.add(pid, prio);
            self.schedule_if_higher_prio(pid, prio);
        } else if old_state == ThreadState::Running {
            // A running thread is only set to a non-running state
            // if it itself initiated it.
            debug_assert_eq!(Some(pid), self.current_pid());

            self.runqueue.pop_head(pid, prio);
            schedule();
        }

        old_state
    }

    /// Returns the state of a thread.
    fn get_state(&self, thread_id: ThreadId) -> Option<ThreadState> {
        if self.is_valid_pid(thread_id) {
            Some(self.threads[usize::from(thread_id)].state)
        } else {
            None
        }
    }

    /// Returns the priority of a thread.
    fn get_priority(&self, thread_id: ThreadId) -> Option<RunqueueId> {
        self.is_valid_pid(thread_id)
            .then(|| self.get_unchecked(thread_id).prio)
    }

    /// Changes the priority of a thread and triggers the scheduler if needed.
    fn set_priority(&mut self, thread_id: ThreadId, prio: RunqueueId) {
        if !self.is_valid_pid(thread_id) {
            return;
        }
        let thread = self.get_unchecked_mut(thread_id);
        let old_prio = thread.prio;
        if old_prio == prio {
            return;
        }
        thread.prio = prio;
        if thread.state != ThreadState::Running {
            // No runqueue changes or scheduler invocation needed.
            return;
        }

        if self.runqueue.peek_head(old_prio) == Some(thread_id) {
            self.runqueue.pop_head(thread_id, old_prio);
        } else {
            self.runqueue.del(thread_id);
        }
        self.runqueue.add(thread_id, prio);

        // Check if the thread is currently running and trigger the scheduler if
        // its prio decreased and another thread might have a higher prio now.
        if self.is_running(thread_id) {
            if prio < old_prio {
                schedule();
            }
            return;
        }

        // Thread isn't running.
        // Only schedule if the thread has a higher priority than the running one.
        if prio > old_prio {
            self.schedule_if_higher_prio(thread_id, prio);
        }
    }

    /// Triggers the scheduler if the thread has a higher priority than the currently running thread.
    fn schedule_if_higher_prio(&mut self, _thread_id: ThreadId, prio: RunqueueId) {
        match self.current().map(|t| t.prio) {
            Some(curr_prio) if curr_prio < prio => schedule(),
            _ => {}
        }
    }

    /// Returns the info if the thread is currently running.
    fn is_running(&self, thread_id: ThreadId) -> bool {
        self.current_pid().is_some_and(|pid| (pid == thread_id))
    }
}

/// Starts threading.
///
/// Supposed to be started early on by OS startup code.
///
/// # Safety
///
/// This function is crafted to be called at a specific point in the RIOT-rs
/// initialization, by `riot-rs-rt`. Don't call this unless you know you need to.
///
/// Currently it expects at least:
/// - Cortex-M: to be called from the reset handler while MSP is active
pub unsafe fn start_threading() {
    Cpu::start_threading();
}

/// Trait for types that fit into a single register.
///
/// Currently implemented for static references (`&'static T`) and usize.
pub trait Arguable {
    fn into_arg(self) -> usize;
}

impl Arguable for usize {
    fn into_arg(self) -> usize {
        self
    }
}

impl Arguable for () {
    fn into_arg(self) -> usize {
        0
    }
}

/// [`Arguable`] is only implemented on *static* references because the references passed to a
/// thread must be valid for its entire lifetime.
impl<T> Arguable for &'static T {
    fn into_arg(self) -> usize {
        self as *const T as usize
    }
}

/// Low-level function to create a thread that runs
/// `func` with `arg`.
///
/// This sets up the stack for the thread and adds it to
/// the runqueue.
///
/// # Panics
///
/// Panics if more than [`THREADS_NUMOF`] concurrent threads have been created.
pub fn thread_create<T: Arguable + Send>(
    func: fn(arg: T),
    arg: T,
    stack: &'static mut [u8],
    prio: u8,
) -> ThreadId {
    let arg = arg.into_arg();
    unsafe { thread_create_raw(func as usize, arg, stack, prio) }
}

/// Low-level function to create a thread without argument
///
/// # Panics
///
/// Panics if more than [`THREADS_NUMOF`] concurrent threads have been created.
pub fn thread_create_noarg(func: fn(), stack: &'static mut [u8], prio: u8) -> ThreadId {
    unsafe { thread_create_raw(func as usize, 0, stack, prio) }
}

/// Creates a thread, low-level.
///
/// # Safety
///
/// Only use when you know what you are doing.
pub unsafe fn thread_create_raw(
    func: usize,
    arg: usize,
    stack: &'static mut [u8],
    prio: u8,
) -> ThreadId {
    THREADS.with_mut(|mut threads| {
        let thread_id = threads
            .create(func, arg, stack, RunqueueId::new(prio))
            .expect("Max `THREADS_NUMOF` concurrent threads should be created.");
        threads.set_state(thread_id, ThreadState::Running);
        thread_id
    })
}

/// Returns the [`ThreadId`] of the currently active thread.
///
/// Note: when called from ISRs, this will return the thread id of the thread
/// that was interrupted.
pub fn current_pid() -> Option<ThreadId> {
    THREADS.with(|threads| threads.current_pid())
}

/// Checks if a given [`ThreadId`] is valid.
pub fn is_valid_pid(thread_id: ThreadId) -> bool {
    THREADS.with(|threads| threads.is_valid_pid(thread_id))
}

/// Thread cleanup function.
///
/// This gets hooked into a newly created thread stack so it gets called when
/// the thread function returns.
///
/// # Panics
///
/// Panics if this is called outside of a thread context.
#[allow(unused)]
fn cleanup() -> ! {
    THREADS.with_mut(|mut threads| {
        let thread_id = threads.current_pid().unwrap();
        threads.set_state(thread_id, ThreadState::Invalid);
    });

    unreachable!();
}

/// "Yields" to another thread with the same priority.
pub fn yield_same() {
    THREADS.with_mut(|mut threads| {
        let Some(prio) = threads.current().map(|t| t.prio) else {
            return;
        };
        if threads.runqueue.advance(prio) {
            schedule();
        }
    })
}

/// Suspends/ pauses the current thread's execution.
pub fn sleep() {
    THREADS.with_mut(|mut threads| {
        let Some(pid) = threads.current_pid() else {
            return;
        };
        threads.set_state(pid, ThreadState::Paused);
    });
}

/// Wakes up a thread and adds it to the runqueue.
///
/// Returns `false` if no paused thread exists for `thread_id`.
pub fn wakeup(thread_id: ThreadId) -> bool {
    THREADS.with_mut(|mut threads| {
        match threads.get_state(thread_id) {
            Some(ThreadState::Paused) => {}
            _ => return false,
        }
        threads.set_state(thread_id, ThreadState::Running);
        true
    })
}

/// Returns the priority of a thread.
///
/// Returns `None` if this is not a valid thread.
pub fn get_priority(thread_id: ThreadId) -> Option<RunqueueId> {
    THREADS.with_mut(|threads| threads.get_priority(thread_id))
}

/// Changes the priority of a thread.
///
/// This might trigger a context switch.
pub fn set_priority(thread_id: ThreadId, prio: RunqueueId) {
    THREADS.with_mut(|mut threads| threads.set_priority(thread_id, prio))
}

/// Returns the size of the internal structure that holds the
/// a thread's data.
pub fn thread_struct_size() -> usize {
    core::mem::size_of::<Thread>()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_basic() {
        assert_eq!(1, 1);
    }
}
