use crate::{thread_flags::ThreadFlags, Arch, Cpu, RunqueueId, ThreadData, ThreadId};

/// Main struct for holding thread data.
#[derive(Debug)]
pub struct Thread {
    /// Saved stack pointer after context switch.
    #[allow(
        dead_code,
        reason = "sp is used in context-specific scheduler implementation"
    )]
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
    /// Core affinity of the thread.
    #[cfg(feature = "core-affinity")]
    pub core_affinity: crate::CoreAffinity,
}

/// Possible states of a thread
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
    /// Waiting to receive on a [`crate::sync::Channel`], i.e. waiting for the sender.
    ChannelRxBlocked(usize),
    /// Waiting to send on a [`crate::sync::Channel`], i.e. waiting for the receiver.
    ChannelTxBlocked(usize),
}

impl Thread {
    /// Creates an empty [`Thread`] object with [`ThreadState::Invalid`].
    pub const fn default() -> Thread {
        Thread {
            sp: 0,
            state: ThreadState::Invalid,
            data: Cpu::DEFAULT_THREAD_DATA,
            flags: 0,
            prio: RunqueueId::new(0),
            pid: ThreadId::new(0),
            #[cfg(feature = "core-affinity")]
            core_affinity: crate::CoreAffinity::no_affinity(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_type_sizes() {
        // `ThreadData` is arch-specific, and is replaced with a dummy value is tests; its size is
        // non-zero otherwise.
        assert_eq!(size_of::<ThreadData>(), 0);
        assert_eq!(
            size_of::<Thread>(),
            size_of::<usize>() + size_of::<ThreadData>() + 24
        );
    }
}
