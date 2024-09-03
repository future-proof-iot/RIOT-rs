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

#[cfg(feature = "multicore")]
mod smp;

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

#[cfg(feature = "core-affinity")]
pub use smp::CoreAffinity;
#[cfg(feature = "multicore")]
pub use smp::CoreId;

#[doc(hidden)]
pub use arch::schedule;

use arch::{Arch, Cpu, ThreadData};
use ensure_once::EnsureOnce;
use riot_rs_runqueue::RunQueue;
use thread::{Thread, ThreadState};

#[cfg(feature = "multicore")]
use smp::{schedule_on_core, Multicore};
#[cfg(feature = "multicore")]
use static_cell::ConstStaticCell;

/// Dummy type that is needed because [`CoreAffinity`] is part of the general API.
///
/// To configure core affinities for threads, the `core-affinity` feature must be enabled.
#[cfg(not(feature = "core-affinity"))]
pub struct CoreAffinity {
    // Phantom field to ensure that `CoreAffinity` can never be constructed by a user.
    _phantom: core::marker::PhantomData<()>,
}

/// The number of possible priority levels.
pub const SCHED_PRIO_LEVELS: usize = 12;

/// The maximum number of concurrent threads that can be created.
pub const THREADS_NUMOF: usize = 16;

#[cfg(feature = "multicore")]
pub const CORES_NUMOF: usize = smp::Chip::CORES as usize;
#[cfg(feature = "multicore")]
pub const IDLE_THREAD_STACK_SIZE: usize = smp::Chip::IDLE_THREAD_STACK_SIZE;

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

    /// The currently running thread(s).
    #[cfg(feature = "multicore")]
    current_threads: [Option<ThreadId>; CORES_NUMOF],
    #[cfg(not(feature = "multicore"))]
    current_thread: Option<ThreadId>,
}

impl Threads {
    const fn new() -> Self {
        Self {
            runqueue: RunQueue::new(),
            threads: [const { Thread::default() }; THREADS_NUMOF],
            thread_blocklist: [const { None }; THREADS_NUMOF],
            #[cfg(feature = "multicore")]
            current_threads: [None; CORES_NUMOF],
            #[cfg(not(feature = "multicore"))]
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
        self.current_pid()
            .map(|pid| &mut self.threads[usize::from(pid)])
    }

    /// Returns the ID of the current thread, or [`None`] if no thread is currently
    /// running.
    ///
    /// On multicore, it returns the ID of the thread that is running on the
    /// current core.
    fn current_pid(&self) -> Option<ThreadId> {
        #[cfg(feature = "multicore")]
        {
            self.current_threads[usize::from(core_id())]
        }
        #[cfg(not(feature = "multicore"))]
        {
            self.current_thread
        }
    }

    /// Returns a mutable reference to the current thread ID, or [`None`]
    /// if no thread is currently running.
    ///
    /// On multicore, it refers to the ID of the thread that is running on the
    /// current core.
    #[allow(dead_code, reason = "used in scheduler implementation")]
    fn current_pid_mut(&mut self) -> &mut Option<ThreadId> {
        #[cfg(feature = "multicore")]
        {
            &mut self.current_threads[usize::from(core_id())]
        }
        #[cfg(not(feature = "multicore"))]
        {
            &mut self.current_thread
        }
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
        _core_affinity: Option<CoreAffinity>,
    ) -> Option<ThreadId> {
        let (thread, pid) = self.get_unused()?;
        Cpu::setup_stack(thread, stack, func, arg);
        thread.prio = prio;
        thread.pid = pid;
        #[cfg(feature = "core-affinity")]
        {
            thread.core_affinity = _core_affinity.unwrap_or_default();
        }

        Some(pid)
    }

    /// Returns immutable access to any thread data.
    ///
    /// # Panics
    ///
    /// Panics if `thread_id` is >= [`THREADS_NUMOF`].
    /// If the thread for this `thread_id` is in an invalid state, the
    /// data in the returned [`Thread`] is undefined, i.e. empty or outdated.
    #[allow(dead_code, reason = "used in scheduler implementation")]
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
    /// This function will also take care of adding/ removing a thread from
    /// the runqueue depending on it's previous and new state.
    ///
    /// # Panics
    ///
    /// Panics if `pid` is >= [`THREADS_NUMOF`].
    fn set_state(&mut self, pid: ThreadId, state: ThreadState) -> ThreadState {
        let thread = self.get_unchecked_mut(pid);
        let old_state = core::mem::replace(&mut thread.state, state);
        let prio = thread.prio;
        match (old_state, state) {
            (old, new) if new == old => {}
            (ThreadState::Invalid, ThreadState::Running) if self.current_pid().is_none() => {
                // `current_pid` is only `None` if we're in start-up phase.
                // The scheduler will be triggered in `start_threading` after all threads have
                // been created.
                self.runqueue.add(pid, prio);
            }
            (_, ThreadState::Running) => {
                #[cfg(feature = "multicore")]
                if self.current_threads.contains(&Some(pid)) {
                    // If the thread is in `current_threads`, it must have been set to a
                    // blocked state before but the scheduler didn't have a chance to run yet.
                    // No further action is needed because in this case a
                    // schedule call is already pending.
                    // The scheduler will re-add the thread to the runqueue in `sched`.
                    return old_state;
                }
                self.runqueue.add(pid, prio);
                self.schedule_if_higher_prio(pid, prio);
            }
            (ThreadState::Running, _) => {
                // A running thread is only set into a non-running state
                // if it itself initiated it.
                debug_assert_eq!(Some(pid), self.current_pid());

                // On multicore, the currently running thread is not in the runqueue
                // anyway, so we don't need to remove it here.
                #[cfg(not(feature = "multicore"))]
                self.runqueue.pop_head(pid, prio);

                schedule()
            }
            _ => {}
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
    fn get_priority(&self, thread_id: ThreadId) -> RunqueueId {
        self.get_unchecked(thread_id).prio
    }

    /// Change the priority of a thread and triggers the scheduler if needed.
    fn set_priority(&mut self, thread_id: ThreadId, new_prio: RunqueueId) {
        if !self.is_valid_pid(thread_id) {
            return;
        }
        let Thread { prio, state, .. } = self.get_unchecked_mut(thread_id);
        let old_prio = *prio;
        if old_prio == new_prio {
            return;
        }
        *prio = new_prio;
        if *state != ThreadState::Running {
            return;
        }

        #[cfg(feature = "multicore")]
        if let Some(core) = self
            .current_threads
            .iter()
            .position(|pid| *pid == Some(thread_id))
        {
            // Only need to trigger the scheduler if the prio decreased and another thread might
            // now have higher priority.
            if new_prio < old_prio {
                schedule_on_core(CoreId(core as u8));
            }
            // The scheduler will re-add the thread with the new prio to the runqueue
            // in `sched`.
            return;
        }
        // Update runqueue.
        if self.runqueue.peek_head(old_prio) == Some(thread_id) {
            self.runqueue.pop_head(thread_id, old_prio);
        } else {
            self.runqueue.del(thread_id);
        }
        self.runqueue.add(thread_id, new_prio);

        #[cfg(not(feature = "multicore"))]
        if self.current_thread == Some(thread_id) {
            // Only need to trigger the scheduler if the prio decreased and another thread might
            // now have higher priority.
            if new_prio < old_prio {
                schedule();
            }
            return;
        }
        // Thread is not currently running.
        // Only scheduler if the thread has a higher priority than a running one.
        if new_prio > old_prio {
            self.schedule_if_higher_prio(thread_id, new_prio);
        }
    }

    /// Trigger the scheduler if the thread has higher priority than (one of0
    /// the running thread(s).
    fn schedule_if_higher_prio(&mut self, _thread_id: ThreadId, prio: RunqueueId) {
        #[cfg(not(feature = "multicore"))]
        if self.current().map(|t| t.prio) < Some(prio) {
            schedule()
        }
        #[cfg(feature = "multicore")]
        {
            let (core, lowest_prio) = self.lowest_running_prio(_thread_id);
            if lowest_prio < Some(prio) {
                schedule_on_core(core);
            }
        }
    }

    /// Adds the thread that is running on the current core to the
    /// runqueue if it's in [`ThreadState::Running`].
    #[cfg(feature = "multicore")]
    fn add_current_thread_to_rq(&mut self) {
        let Some(thread) = self.current() else {
            return;
        };
        if thread.state == ThreadState::Running {
            let prio = thread.prio;
            let pid = thread.pid;
            self.runqueue.add(pid, prio);
        }
    }

    /// Returns the next thread from the runqueue.
    ///
    /// On single-core, it only reads the head of the runqueue without
    /// removing the thread.
    ///
    /// On multi-core, the thread is removed from the runqueue to avoid that
    /// the same thread is returned multiple times.
    ///
    /// If core-affinities are enabled, the head of the runqueue is only returned if
    /// its affinity matches the current core. Otherwise the runqueue is iterated until
    /// a thread with matching affinity is found.
    #[allow(dead_code, reason = "used in scheduler implementation")]
    fn get_next_pid(&mut self) -> Option<ThreadId> {
        #[cfg(not(feature = "multicore"))]
        {
            // Read head of runqueue.
            self.runqueue.get_next()
        }
        #[cfg(feature = "multicore")]
        #[cfg(not(feature = "core-affinity"))]
        {
            // Pop head of runqueue.
            self.runqueue.pop_next()
        }
        #[cfg(feature = "multicore")]
        #[cfg(feature = "core-affinity")]
        {
            // Iterate through the runqueue until a thread with matching core
            // affinity is found.
            let (mut next, prio) = self.runqueue.peek_next()?;
            if !self.is_affine_to_curr_core(next) {
                let iter = self.runqueue.iter_from(next, prio);
                next = iter
                    .filter(|pid| self.is_affine_to_curr_core(*pid))
                    .next()?
            }
            // Delete thread from runqueue to match the `pop_next`.
            self.runqueue.del(next);
            Some(next)
        }
    }

    /// Searches for the lowest priority thread among the currently running threads.
    ///
    /// Returns the core that the lowest priority thread is running on, and its priority.
    /// Returns `None` for the priority if an idle core was found, which is only the case
    /// during startup.
    ///
    /// If core-affinities are enabled, the parameter `_pid` restricts the search to only
    /// consider the cores that match this thread's [`CoreAffinity`].
    #[cfg(feature = "multicore")]
    fn lowest_running_prio(&self, _pid: ThreadId) -> (CoreId, Option<RunqueueId>) {
        #[cfg(feature = "core-affinity")]
        let affinity = self.get_unchecked(_pid).core_affinity;
        // Find the lowest priority thread among the currently running threads.
        self.current_threads
            .iter()
            .enumerate()
            .filter_map(|(core, pid)| {
                let core = CoreId(core as u8);
                // Skip cores that don't match the core-affinity.
                #[cfg(feature = "core-affinity")]
                if !affinity.contains(core) {
                    return None;
                }
                let prio = pid.map(|pid| self.get_unchecked(pid).prio);
                Some((core, prio))
            })
            .min_by_key(|(_, rq)| *rq)
            .unwrap()
    }

    /// Checks if a thread can be scheduled on the current core.
    #[allow(dead_code, reason = "used in scheduler implementation")]
    #[cfg(feature = "core-affinity")]
    fn is_affine_to_curr_core(&self, pid: ThreadId) -> bool {
        self.get_unchecked(pid)
            .core_affinity
            .contains(crate::core_id())
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
    #[cfg(feature = "multicore")]
    {
        // Idle thread that prompts the core to enter deep sleep.
        fn idle_thread() {
            loop {
                Cpu::wfi();
            }
        }

        // Stacks for the idle threads.
        // Creating them inside the below for-loop is not possible because it would result in
        // duplicate identifiers for the created `static`.
        static STACKS: [ConstStaticCell<[u8; IDLE_THREAD_STACK_SIZE]>; CORES_NUMOF] =
            [const { ConstStaticCell::new([0u8; IDLE_THREAD_STACK_SIZE]) }; CORES_NUMOF];

        // Create one idle thread for each core with lowest priority.
        for stack in &STACKS {
            thread_create_noarg(idle_thread, stack.take(), 0, None);
        }

        smp::Chip::startup_other_cores();
    }
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
    core_affinity: Option<CoreAffinity>,
) -> ThreadId {
    let arg = arg.into_arg();
    unsafe { thread_create_raw(func as usize, arg, stack, prio, core_affinity) }
}

/// Low-level function to create a thread without argument
///
/// # Panics
///
/// Panics if more than [`THREADS_NUMOF`] concurrent threads have been created.
pub fn thread_create_noarg(
    func: fn(),
    stack: &'static mut [u8],
    prio: u8,
    core_affinity: Option<CoreAffinity>,
) -> ThreadId {
    unsafe { thread_create_raw(func as usize, 0, stack, prio, core_affinity) }
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
    core_affinity: Option<CoreAffinity>,
) -> ThreadId {
    THREADS.with_mut(|mut threads| {
        let thread_id = threads
            .create(func, arg, stack, RunqueueId::new(prio), core_affinity)
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

/// Returns the id of the CPU that this thread is running on.
#[cfg(feature = "multicore")]
pub fn core_id() -> CoreId {
    smp::Chip::core_id()
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
    if THREADS.with_mut(|mut threads| {
        let Some(rq) = threads.current().map(|t| t.prio) else {
            return false;
        };
        #[cfg(not(feature = "multicore"))]
        {
            threads.runqueue.advance(rq)
        }
        #[cfg(feature = "multicore")]
        {
            // On multicore, the current thread is removed from the runqueue, and then
            // re-added **at the tail** in `sched` the next time the scheduler is triggered.
            // Simply triggering the scheduler therefore implicitly advances the runqueue.
            !threads.runqueue.is_empty(rq)
        }
    }) {
        schedule();
    }
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
    THREADS.with_mut(|threads| {
        threads
            .is_valid_pid(thread_id)
            .then(|| threads.get_priority(thread_id))
    })
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
