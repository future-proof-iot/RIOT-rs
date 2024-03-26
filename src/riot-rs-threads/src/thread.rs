use crate::{thread_flags::ThreadFlags, Arch, Cpu, RunqueueId, ThreadData, ThreadId};

/// Main struct for holding thread data.
#[derive(Debug)]
pub struct Thread {
    /// Saved stack pointer after context switch.
    pub sp: usize,
    /// The thread's current state.
    pub state: ThreadState,
    /// Priority of the thread between 0..[`super::SCHED_PRIO_LEVELS`].
    /// Multiple threads may have the same priority.
    pub prio: RunqueueId,
    /// Id of the thread between 0..[`super::THREADS_NUMOF`].
    /// Ids are unique while a thread is alive but reused after a thread finished.
    pub pid: ThreadId,
    /// Flags set for the thread.
    pub flags: ThreadFlags,
    /// Arch-specific thread data.
    #[allow(dead_code)]
    pub(crate) data: ThreadData,
}

/// Possible states of a thread
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ThreadState {
    /// No active thread.
    Invalid,
    /// Ready to run.
    ///
    /// This doesn't necessarily mean that the thread is currently running,
    /// but rather that it is in the runqueue.
    Running,
    /// Suspended / paused.
    Paused,
    /// Waiting to acquire a [`super::lock::Lock`].
    LockBlocked,
    /// Waiting for [`ThreadFlags`] to be set.
    FlagBlocked(crate::thread_flags::WaitMode),
    /// Waiting to receive on a [`super::channel::Channel`], i.e. waiting for the sender.
    ChannelRxBlocked(usize),
    /// Waiting to send on a [`super::channel::Channel`], i.e. waiting for the receiver.
    ChannelTxBlocked(usize),
    Zombie,
}

impl Thread {
    /// Creates an empty [`Thread`] object with [`ThreadState::Invalid`].
    pub const fn default() -> Thread {
        Thread {
            sp: 0,
            state: ThreadState::Invalid,
            data: Cpu::DEFAULT_THREAD_DATA,
            flags: 0,
            prio: 0,
            pid: 0,
        }
    }
}
