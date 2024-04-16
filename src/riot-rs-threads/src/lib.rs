#![cfg_attr(not(test), no_std)]
#![feature(inline_const)]
#![feature(naked_functions)]
#![feature(used_with_arg)]
// Disable indexing lints for now, possible panics are documented or rely on internally-enforced
// invariants
#![allow(clippy::indexing_slicing)]

use critical_section::CriticalSection;

use riot_rs_runqueue::RunQueue;
pub use riot_rs_runqueue::{RunqueueId, ThreadId};

mod arch;
mod ensure_once;
mod thread;
mod threadlist;

pub mod channel;
pub mod lock;
pub mod thread_flags;

pub use arch::schedule;
pub use thread::{Thread, ThreadState};
pub use thread_flags as flags;
pub use threadlist::ThreadList;

use arch::{Arch, Cpu, ThreadData};
use ensure_once::EnsureOnce;

/// a global defining the number of possible priority levels
pub const SCHED_PRIO_LEVELS: usize = 12;

/// a global defining the number of threads that can be created
pub const THREADS_NUMOF: usize = 16;

pub(crate) static THREADS: EnsureOnce<Threads> = EnsureOnce::new(Threads::new());

pub type ThreadFn = fn();

#[linkme::distributed_slice]
pub static THREAD_FNS: [ThreadFn] = [..];

/// Struct holding all scheduler state
pub struct Threads {
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
    pub(crate) fn current(&mut self) -> Option<&mut Thread> {
        self.current_thread
            .map(|tid| &mut self.threads[tid as usize])
    }

    pub fn current_pid(&self) -> Option<ThreadId> {
        self.current_thread
    }

    /// Creates a new thread.
    ///
    /// This sets up the stack and TCB for this thread.
    ///
    /// Returns `None` if there is no free thread slot.
    pub(crate) fn create(
        &mut self,
        func: usize,
        arg: usize,
        stack: &'static mut [u8],
        prio: u8,
    ) -> Option<&mut Thread> {
        if let Some((thread, pid)) = self.get_unused() {
            thread.sp = Cpu::setup_stack(stack, func, arg);
            thread.prio = prio;
            thread.pid = pid;
            thread.state = ThreadState::Paused;

            Some(thread)
        } else {
            None
        }
    }

    // fn get_unchecked(&self, thread_id: ThreadId) -> &Thread {
    //     &self.threads[thread_id as usize]
    // }

    /// Returns mutable access to any thread data.
    ///
    /// # Panics
    ///
    /// Panics if `thread_id` is >= [`THREADS_NUMOF`].
    /// If the thread for this `thread_id` is in an invalid state, the
    /// data in the returned [`Thread`] is undefined, i.e. empty or outdated.
    fn get_unchecked_mut(&mut self, thread_id: ThreadId) -> &mut Thread {
        &mut self.threads[thread_id as usize]
    }

    /// Returns an unused ThreadId / Thread slot.
    fn get_unused(&mut self) -> Option<(&mut Thread, ThreadId)> {
        for i in 0..THREADS_NUMOF {
            if self.threads[i].state == ThreadState::Invalid {
                return Some((&mut self.threads[i], i as ThreadId));
            }
        }
        None
    }

    /// Checks if a thread with valid state exists for this `thread_id`.
    fn is_valid_pid(&self, thread_id: ThreadId) -> bool {
        if thread_id as usize >= THREADS_NUMOF {
            false
        } else {
            self.threads[thread_id as usize].state != ThreadState::Invalid
        }
    }

    /// Sets the state of a thread.
    ///
    /// This function handles adding/ removing the thread to the Runqueue depending
    /// on its previous or new state.
    ///
    /// # Panics
    ///
    /// Panics if `pid` is >= [`THREADS_NUMOF`].
    pub(crate) fn set_state(&mut self, pid: ThreadId, state: ThreadState) -> ThreadState {
        let thread = &mut self.threads[pid as usize];
        let old_state = thread.state;
        thread.state = state;
        if old_state != ThreadState::Running && state == ThreadState::Running {
            self.runqueue.add(thread.pid, thread.prio);
        } else if old_state == ThreadState::Running && state != ThreadState::Running {
            self.runqueue.del(thread.pid, thread.prio);
        }

        old_state
    }

    /// Returns the state of a thread.
    pub fn get_state(&self, thread_id: ThreadId) -> Option<ThreadState> {
        if self.is_valid_pid(thread_id) {
            Some(self.threads[thread_id as usize].state)
        } else {
            None
        }
    }
}

/// Starts threading.
///
/// Supposed to be started early on by OS startup code.
///
/// # Safety
///
/// This may only be called once.
///
/// # Panics
///
/// Panics if no thread exists.
pub unsafe fn start_threading() {
    // faking a critical section to get THREADS
    // SAFETY: caller ensures invariants
    let cs = unsafe { CriticalSection::new() };
    let next_sp = THREADS.with_mut_cs(cs, |mut threads| {
        let next_pid = threads.runqueue.get_next().unwrap();
        threads.threads[next_pid as usize].sp
    });
    Cpu::start_threading(next_sp);
}

/// Trait for types that fit into a single register.
///
/// Currently implemented for references (`&T`) and usize.
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

impl<T> Arguable for &T {
    fn into_arg(self) -> usize {
        self as *const T as usize
    }
}

/// Low-level function to create a thread that runs
/// `func` with `arg`.
///
/// This sets up the stack for the thread and adds it to
/// the runqueue.
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
pub fn thread_create_noarg(func: fn(), stack: &'static mut [u8], prio: u8) -> ThreadId {
    unsafe { thread_create_raw(func as usize, 0, stack, prio) }
}

/// Creates a thread, low-level.
///
/// # Safety
/// only use when you know what you are doing.
pub unsafe fn thread_create_raw(
    func: usize,
    arg: usize,
    stack: &'static mut [u8],
    prio: u8,
) -> ThreadId {
    THREADS.with_mut(|mut threads| {
        let thread_id = threads.create(func, arg, stack, prio).unwrap().pid;
        threads.set_state(thread_id, ThreadState::Running);
        thread_id
    })
}

/// Returns the [`ThreadState`] for this `thread_id`.
///
/// Returns `None` if `thread_id` is out of bound or no thread with
/// valid state exists.
pub fn get_state(thread_id: ThreadId) -> Option<ThreadState> {
    THREADS.with(|threads| threads.get_state(thread_id))
}

/// Returns the [`ThreadId`] of the currently active thread.
///
/// Note: when called from ISRs, this will return the thread id of the thread
/// that was interrupted.
pub fn current_pid() -> Option<ThreadId> {
    THREADS.with(|threads| threads.current_pid())
}

/// Checks if a given [`ThreadId`] is valid
pub fn is_valid_pid(thread_id: ThreadId) -> bool {
    THREADS.with(|threads| threads.is_valid_pid(thread_id))
}

/// Thread cleanup function.
///
/// This gets hooked into a newly created thread stack so it gets called when
/// the thread function returns.
#[allow(unused)]
fn cleanup() -> ! {
    THREADS.with_mut(|mut threads| {
        let thread_id = threads.current_pid().unwrap();
        threads.set_state(thread_id, ThreadState::Invalid);
    });

    schedule();

    unreachable!();
}

/// "Yields" to another thread with the same priority.
pub fn yield_same() {
    THREADS.with_mut(|mut threads| {
        let runqueue = threads.current().unwrap().prio;
        threads.runqueue.advance(runqueue);
        schedule();
    })
}

/// Suspends/ pauses the current thread's execution.
pub fn sleep() {
    THREADS.with_mut(|mut threads| {
        let pid = threads.current_pid().unwrap();
        threads.set_state(pid, ThreadState::Paused);
        schedule();
    });
}

/// Wakes up a thread and adds it to the runqueue.
///
/// Returns `false` if no paused thread exists for `thread_id`.
pub fn wakeup(thread_id: ThreadId) -> bool {
    THREADS.with_mut(|mut threads| {
        if let Some(state) = threads.get_state(thread_id) {
            if state == ThreadState::Paused {
                threads.set_state(thread_id, ThreadState::Running);
                schedule();
                true
            } else {
                false
            }
        } else {
            false
        }
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_basic() {
        assert_eq!(1, 1);
    }
}
