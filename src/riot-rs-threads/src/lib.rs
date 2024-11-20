//! Multi-threading for Ariel OS.
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

#[cfg(feature = "multi-core")]
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

#[doc(hidden)]
pub mod events {
    use crate::sync::Event;
    // this is set in `riot_rs_embassy::init_task()`
    pub static THREAD_START_EVENT: Event = Event::new();
}

pub use riot_rs_runqueue::{RunqueueId, ThreadId};
pub use thread_flags as flags;

#[cfg(feature = "core-affinity")]
pub use smp::CoreAffinity;

use arch::{schedule, Arch, Cpu, ThreadData};
use ensure_once::EnsureOnce;
use riot_rs_runqueue::RunQueue;
use thread::{Thread, ThreadState};

#[cfg(feature = "multi-core")]
use smp::{schedule_on_core, Multicore};
#[cfg(feature = "multi-core")]
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

#[cfg(feature = "multi-core")]
pub const CORES_NUMOF: usize = smp::Chip::CORES as usize;
#[cfg(feature = "multi-core")]
pub const IDLE_THREAD_STACK_SIZE: usize = smp::Chip::IDLE_THREAD_STACK_SIZE;

static SCHEDULER: EnsureOnce<Scheduler> = EnsureOnce::new(Scheduler::new());

pub type ThreadFn = fn();

#[linkme::distributed_slice]
pub static THREAD_FNS: [ThreadFn] = [..];

/// Struct holding all scheduler state
struct Scheduler {
    /// Global thread runqueue.
    runqueue: RunQueue<SCHED_PRIO_LEVELS, THREADS_NUMOF>,
    /// The actual TCBs.
    threads: [Thread; THREADS_NUMOF],
    /// `Some` when a thread is blocking another thread due to conflicting
    /// resource access.
    thread_blocklist: [Option<ThreadId>; THREADS_NUMOF],

    /// The currently running thread(s).
    #[cfg(feature = "multi-core")]
    current_threads: [Option<ThreadId>; CORES_NUMOF],
    #[cfg(not(feature = "multi-core"))]
    current_thread: Option<ThreadId>,
}

impl Scheduler {
    const fn new() -> Self {
        Self {
            runqueue: RunQueue::new(),
            threads: [const { Thread::default() }; THREADS_NUMOF],
            thread_blocklist: [const { None }; THREADS_NUMOF],
            #[cfg(feature = "multi-core")]
            current_threads: [None; CORES_NUMOF],
            #[cfg(not(feature = "multi-core"))]
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
    /// On multi-core, it returns the ID of the thread that is running on the
    /// current core.
    #[inline]
    fn current_pid(&self) -> Option<ThreadId> {
        #[cfg(feature = "multi-core")]
        {
            self.current_threads[usize::from(core_id())]
        }
        #[cfg(not(feature = "multi-core"))]
        {
            self.current_thread
        }
    }

    /// Returns a mutable reference to the current thread ID, or [`None`]
    /// if no thread is currently running.
    ///
    /// On multi-core, it refers to the ID of the thread that is running on the
    /// current core.
    #[allow(dead_code, reason = "used in scheduler implementation")]
    fn current_pid_mut(&mut self) -> &mut Option<ThreadId> {
        #[cfg(feature = "multi-core")]
        {
            &mut self.current_threads[usize::from(core_id())]
        }
        #[cfg(not(feature = "multi-core"))]
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
        thread.state = ThreadState::Paused;
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

            // On multi-core, the currently running thread is not in the runqueue
            // anyway, so we don't need to remove it here.
            #[cfg(not(feature = "multi-core"))]
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

        // Check if the thread is among the current threads and trigger scheduler if
        // its prio decreased and another thread might have a higher prio now.
        // This has to be done on multi-core **before the runqueue changes below**, because
        // a currently running thread is not in the runqueue and therefore the runqueue changes
        // should not be applied.
        #[cfg(feature = "multi-core")]
        match self.is_running(thread_id) {
            Some(core) if prio < old_prio => return schedule_on_core(CoreId(core as u8)),
            Some(_) => return,
            _ => {}
        }

        // Update the runqueue.
        if self.runqueue.peek_head(old_prio) == Some(thread_id) {
            self.runqueue.pop_head(thread_id, old_prio);
        } else {
            self.runqueue.del(thread_id);
        }
        self.runqueue.add(thread_id, prio);

        // Check & handle if the thread is among the current threads for single-core,
        // analogous to the above multi-core implementation.
        #[cfg(not(feature = "multi-core"))]
        match self.is_running(thread_id) {
            Some(_) if prio < old_prio => return schedule(),
            Some(_) => return,
            _ => {}
        }

        // Thread isn't running.
        // Only schedule if the thread has a higher priority than a running one.
        if prio > old_prio {
            self.schedule_if_higher_prio(thread_id, prio);
        }
    }

    /// Triggers the scheduler if the thread has a higher priority than (one of)
    /// the running thread(s).
    fn schedule_if_higher_prio(&mut self, _thread_id: ThreadId, prio: RunqueueId) {
        #[cfg(not(feature = "multi-core"))]
        match self.current().map(|t| t.prio) {
            Some(curr_prio) if curr_prio < prio => schedule(),
            _ => {}
        }
        #[cfg(feature = "multi-core")]
        match self.lowest_running_prio(_thread_id) {
            (core, Some(lowest_prio)) if lowest_prio < prio => schedule_on_core(core),
            _ => {}
        }
    }

    /// Returns `Some` if the thread is currently running on a core.
    ///
    /// On multi-core, the core-id is returned as usize, on single-core
    /// the usize is always 0.
    fn is_running(&self, thread_id: ThreadId) -> Option<usize> {
        #[cfg(not(feature = "multi-core"))]
        {
            self.current_pid()
                .filter(|pid| *pid == thread_id)
                .map(|_| 0)
        }

        #[cfg(feature = "multi-core")]
        {
            self.current_threads
                .iter()
                .position(|pid| *pid == Some(thread_id))
        }
    }

    /// Adds the thread that is running on the current core to the
    /// runqueue if it has state [`ThreadState::Running`].
    #[cfg(feature = "multi-core")]
    #[allow(dead_code, reason = "used in scheduler implementation")]
    fn add_current_thread_to_rq(&mut self) {
        let (pid, prio) = match self.current() {
            Some(&mut Thread {
                pid,
                prio,
                state: ThreadState::Running,
                ..
            }) => (pid, prio),
            _ => return,
        };
        self.runqueue.add(pid, prio);
    }

    /// Returns the next thread from the runqueue.
    ///
    /// On single-core, the thread remains in the runqueue, so subsequent calls
    /// will return the same thread.
    ///
    /// On multi-core, the thread is removed so that subsequent calls will each
    /// return a different thread. This prevents that a thread is picked multiple
    /// times by the scheduler when it is invoked on different cores.
    #[allow(dead_code, reason = "used in scheduler implementation")]
    fn get_next_pid(&mut self) -> Option<ThreadId> {
        // On single-core, only read the head of the runqueue.
        #[cfg(not(feature = "multi-core"))]
        {
            self.runqueue.get_next()
        }

        // On multi-core, the head is popped of the runqueue.
        #[cfg(all(feature = "multi-core", not(feature = "core-affinity")))]
        {
            self.runqueue.pop_next()
        }

        // On multi-core with core-affinities, get next thread with matching affinity.
        #[cfg(all(feature = "multi-core", feature = "core-affinity"))]
        {
            // TODO: this would benefit from a `del_one_with_filter` to avoid
            // iterating twice.
            let next = self
                .runqueue
                .get_next_filter(|&t| self.is_affine_to_curr_core(t))?;
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
    #[cfg(feature = "multi-core")]
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
                Some(((core), prio))
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

/// ID of a physical core.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CoreId(pub(crate) u8);

impl From<CoreId> for usize {
    fn from(value: CoreId) -> Self {
        value.0 as usize
    }
}

/// Starts threading.
///
/// Supposed to be started early on by OS startup code.
///
/// # Safety
///
/// This function is crafted to be called at a specific point in the Ariel OS
/// initialization, by `ariel-os-rt`. Don't call this unless you know you need to.
///
/// Currently it expects at least:
/// - Cortex-M: to be called from the reset handler while MSP is active
pub unsafe fn start_threading() {
    #[cfg(feature = "multi-core")]
    {
        riot_rs_debug::log::debug!("ariel-os-threads: SMP mode with {} cores", CORES_NUMOF);

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
    SCHEDULER.with_mut(|mut scheduler| {
        let thread_id = scheduler
            .create(func, arg, stack, RunqueueId::new(prio), core_affinity)
            .expect("Max `THREADS_NUMOF` concurrent threads should be created.");
        scheduler.set_state(thread_id, ThreadState::Running);
        thread_id
    })
}

/// Returns the [`ThreadId`] of the currently active thread.
///
/// Note: when called from ISRs, this will return the thread id of the thread
/// that was interrupted.
pub fn current_pid() -> Option<ThreadId> {
    SCHEDULER.with(|scheduler| scheduler.current_pid())
}

/// Returns the id of the CPU that this thread is running on.
pub fn core_id() -> CoreId {
    #[cfg(not(feature = "multi-core"))]
    {
        CoreId(0)
    }
    #[cfg(feature = "multi-core")]
    {
        smp::Chip::core_id()
    }
}

/// Checks if a given [`ThreadId`] is valid.
pub fn is_valid_pid(thread_id: ThreadId) -> bool {
    SCHEDULER.with(|scheduler| scheduler.is_valid_pid(thread_id))
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
    SCHEDULER.with_mut(|mut scheduler| {
        let thread_id = scheduler.current_pid().unwrap();
        scheduler.set_state(thread_id, ThreadState::Invalid);
    });

    unreachable!();
}

/// "Yields" to another thread with the same priority.
pub fn yield_same() {
    SCHEDULER.with_mut(|mut scheduler| {
        let Some(&mut Thread {
            prio,
            pid: _pid,
            #[cfg(feature = "core-affinity")]
                core_affinity: _affinity,
            ..
        }) = scheduler.current()
        else {
            return;
        };

        #[cfg(not(feature = "multi-core"))]
        if scheduler.runqueue.advance(prio) {
            schedule()
        }

        // On multi-core, the current thread is removed from the runqueue, and then
        // re-added **at the tail** in `sched` the next time the scheduler is invoked.
        // Simply triggering the scheduler therefore implicitly advances the runqueue.
        #[cfg(feature = "multi-core")]
        if !scheduler.runqueue.is_empty(prio) {
            schedule();

            // Check if the yielding thread can continue their execution on another
            // core that currently runs a lower priority thread.
            // This is only necessary when core-affinities are enabled, because only
            // then it is possible that a lower prio thread runs while a higher prio
            // runqueue isn't empty.
            #[cfg(feature = "core-affinity")]
            if _affinity == CoreAffinity::no_affinity() {
                scheduler.schedule_if_higher_prio(_pid, prio);
            }
        }
    })
}

/// Suspends/ pauses the current thread's execution.
pub fn sleep() {
    SCHEDULER.with_mut(|mut scheduler| {
        let Some(pid) = scheduler.current_pid() else {
            return;
        };
        scheduler.set_state(pid, ThreadState::Paused);
    });
}

/// Wakes up a thread and adds it to the runqueue.
///
/// Returns `false` if no paused thread exists for `thread_id`.
pub fn wakeup(thread_id: ThreadId) -> bool {
    SCHEDULER.with_mut(|mut scheduler| {
        match scheduler.get_state(thread_id) {
            Some(ThreadState::Paused) => {}
            _ => return false,
        }
        scheduler.set_state(thread_id, ThreadState::Running);
        true
    })
}

/// Returns the priority of a thread.
///
/// Returns `None` if this is not a valid thread.
pub fn get_priority(thread_id: ThreadId) -> Option<RunqueueId> {
    SCHEDULER.with_mut(|scheduler| scheduler.get_priority(thread_id))
}

/// Changes the priority of a thread.
///
/// This might trigger a context switch.
pub fn set_priority(thread_id: ThreadId, prio: RunqueueId) {
    SCHEDULER.with_mut(|mut scheduler| scheduler.set_priority(thread_id, prio))
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
