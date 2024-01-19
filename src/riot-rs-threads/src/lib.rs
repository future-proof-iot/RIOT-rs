#![cfg_attr(not(test), no_std)]
#![feature(inline_const)]
#![feature(naked_functions)]
#![feature(used_with_arg)]

use critical_section::CriticalSection;

use riot_rs_runqueue::RunQueue;
pub use riot_rs_runqueue::{RunqueueId, ThreadId};

mod arch;
mod ensure_once;
mod threadlist;

pub mod channel;
pub mod lock;
pub mod thread;
pub mod thread_flags;

pub use arch::schedule;
pub use thread::{Thread, ThreadState};
pub use thread_flags as flags;
pub use threadlist::ThreadList;

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
    /// global thread runqueue
    runqueue: RunQueue<SCHED_PRIO_LEVELS, THREADS_NUMOF>,
    threads: [Thread; THREADS_NUMOF],
    thread_blocklist: [Option<ThreadId>; THREADS_NUMOF],
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

    pub(crate) fn current(&mut self) -> Option<&mut Thread> {
        self.current_thread
            .map(|tid| &mut self.threads[tid as usize])
    }

    pub fn current_pid(&self) -> Option<ThreadId> {
        self.current_thread
    }

    /// Create a new thread
    pub(crate) fn create(
        &mut self,
        func: usize,
        arg: usize,
        stack: &mut [u8],
        prio: u8,
    ) -> Option<&mut Thread> {
        if let Some((thread, pid)) = self.get_unused() {
            thread.sp = arch::setup_stack(stack, func, arg);
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

    fn get_unchecked_mut(&mut self, thread_id: ThreadId) -> &mut Thread {
        &mut self.threads[thread_id as usize]
    }

    // get an unused ThreadId / Thread slot
    fn get_unused(&mut self) -> Option<(&mut Thread, ThreadId)> {
        for i in 0..THREADS_NUMOF {
            if self.threads[i].state == ThreadState::Invalid {
                return Some((&mut self.threads[i], i as ThreadId));
            }
        }
        None
    }

    fn is_valid_pid(&self, thread_id: ThreadId) -> bool {
        if thread_id as usize >= THREADS_NUMOF {
            false
        } else {
            self.threads[thread_id as usize].state != ThreadState::Invalid
        }
    }

    /// set state of thread
    ///
    /// This function handles adding/removing the thread to the Runqueue depending
    /// on its previous or new state.
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

    pub fn get_state(&self, thread_id: ThreadId) -> Option<ThreadState> {
        if self.is_valid_pid(thread_id) {
            Some(self.threads[thread_id as usize].state)
        } else {
            None
        }
    }
}

/// start threading
///
/// Supposed to be started early on by OS startup code.
///
/// # Safety
/// This may only be called once.
pub unsafe fn start_threading() {
    // faking a critical section to get THREADS
    let cs = CriticalSection::new();
    let next_sp = THREADS.with_mut_cs(cs, |mut threads| {
        let next_pid = threads.runqueue.get_next().unwrap();
        threads.current_thread = Some(next_pid);
        threads.threads[next_pid as usize].sp
    });
    arch::start_threading(next_sp);
}

/// trait for types that fit into a single register.
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

/// Low-level function to create a thread
pub fn thread_create<T: Arguable + Send>(
    func: fn(arg: T),
    arg: T,
    stack: &mut [u8],
    prio: u8,
) -> ThreadId {
    let arg = arg.into_arg();
    unsafe { thread_create_raw(func as usize, arg, stack, prio) }
}

/// Create a thread, low-level
///
/// # Safety
/// only use when you know what you are doing.
pub unsafe fn thread_create_raw(func: usize, arg: usize, stack: &mut [u8], prio: u8) -> ThreadId {
    THREADS.with_mut(|mut threads| {
        let thread_id = threads.create(func, arg, stack, prio).unwrap().pid;
        threads.set_state(thread_id, ThreadState::Running);
        thread_id
    })
}

pub fn get_state(thread_id: ThreadId) -> Option<ThreadState> {
    THREADS.with(|threads| threads.get_state(thread_id))
}

/// Returns the thread id of the currently active thread.
///
/// Note: when called from ISRs, this will return the thread id of the thread
/// that was interrupted.
pub fn current_pid() -> Option<ThreadId> {
    THREADS.with(|threads| threads.current_pid())
}

/// Check if a given [`ThreadId`] is valid
pub fn is_valid_pid(thread_id: ThreadId) -> bool {
    THREADS.with(|threads| threads.is_valid_pid(thread_id))
}

/// thread cleanup function
///
/// This gets hooked into a newly created thread stack so it gets called when
/// the thread function returns.
fn cleanup() -> ! {
    THREADS.with_mut(|mut threads| {
        let thread_id = threads.current_pid().unwrap();
        threads.set_state(thread_id, ThreadState::Invalid);
    });

    arch::schedule();

    unreachable!();
}

pub fn yield_same() {
    THREADS.with_mut(|mut threads| {
        let runqueue = threads.current().unwrap().prio;
        threads.runqueue.advance(runqueue);
        arch::schedule();
    })
}

/// Suspend/pause the current thread's execution
pub fn sleep() {
    THREADS.with_mut(|mut threads| {
        let pid = threads.current_pid().unwrap();
        threads.set_state(pid, ThreadState::Paused);
        arch::schedule();
    });
}

/// Suspend/pause the current thread's execution
pub fn wakeup(thread_id: ThreadId) -> bool {
    THREADS.with_mut(|mut threads| {
        if let Some(state) = threads.get_state(thread_id) {
            if state == ThreadState::Paused {
                threads.set_state(thread_id, ThreadState::Running);
                arch::schedule();
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
