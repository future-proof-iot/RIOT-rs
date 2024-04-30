//! Thread flags.
use crate::{ThreadId, ThreadState, Threads, THREADS};

/// Bitmask that represent the flags that are set for a thread.
pub type ThreadFlags = u16;

/// Possible waiting modes for [`ThreadFlags`].
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum WaitMode {
    Any(ThreadFlags),
    All(ThreadFlags),
}

/// Sets flags for a thread.
///
/// If the thread was blocked on these flags it's unblocked and added
/// to the runqueue.
///
/// # Panics
///
/// Panics if `thread_id` is >= [`THREADS_NUMOF`](crate::THREADS_NUMOF).
pub fn set(thread_id: ThreadId, mask: ThreadFlags) {
    THREADS.with_mut(|mut threads| threads.flag_set(thread_id, mask));
}

/// Waits until all flags in `mask` are set for the current thread.
///
/// Returns the set flags for this mask and clears them for the thread.
///
/// # Panics
///
/// Panics if this is called outside of a thread context.
pub fn wait_all(mask: ThreadFlags) -> ThreadFlags {
    loop {
        if let Some(flags) = THREADS.with_mut(|mut threads| threads.flag_wait_all(mask)) {
            return flags;
        }
    }
}

/// Waits until any flag in `mask` is set for the current thread.
///
/// Returns all set flags for this mask and clears them for the thread.
///
/// # Panics
///
/// Panics if this is called outside of a thread context.
pub fn wait_any(mask: ThreadFlags) -> ThreadFlags {
    loop {
        if let Some(flags) = THREADS.with_mut(|mut threads| threads.flag_wait_any(mask)) {
            return flags;
        }
    }
}

/// Waits until any flag in `mask` is set for the current thread.
///
/// Compared to [`wait_any`], this returns and clears only one flag
/// from the mask.
///
/// # Panics
///
/// Panics if this is called outside of a thread context.
pub fn wait_one(mask: ThreadFlags) -> ThreadFlags {
    loop {
        if let Some(flags) = THREADS.with_mut(|mut threads| threads.flag_wait_one(mask)) {
            return flags;
        }
    }
}

/// Clears flags for the current thread.
///
/// # Panics
///
/// Panics if this is called outside of a thread context.
pub fn clear(mask: ThreadFlags) -> ThreadFlags {
    THREADS.with_mut(|mut threads| {
        let thread = threads.current().unwrap();
        let res = thread.flags & mask;
        thread.flags &= !mask;
        res
    })
}

/// Returns the flags set for the current thread.
///
/// # Panics
///
/// Panics if this is called outside of a thread context.
pub fn get() -> ThreadFlags {
    // TODO: current() requires us to use mutable `threads` here
    THREADS.with_mut(|mut threads| threads.current().unwrap().flags)
}

impl Threads {
    // thread flags implementation
    fn flag_set(&mut self, thread_id: ThreadId, mask: ThreadFlags) {
        let thread = self.get_unchecked_mut(thread_id);
        thread.flags |= mask;
        if match thread.state {
            ThreadState::FlagBlocked(mode) => match mode {
                WaitMode::Any(bits) => thread.flags & bits != 0,
                WaitMode::All(bits) => thread.flags & bits == bits,
            },
            _ => false,
        } {
            self.set_state(thread_id, ThreadState::Running);
            crate::schedule();
        }
    }

    fn flag_wait_all(&mut self, mask: ThreadFlags) -> Option<ThreadFlags> {
        let thread = self.current().unwrap();
        if thread.flags & mask == mask {
            thread.flags &= !mask;
            Some(mask)
        } else {
            let thread_id = thread.pid;
            self.set_state(thread_id, ThreadState::FlagBlocked(WaitMode::All(mask)));
            crate::schedule();
            None
        }
    }

    fn flag_wait_any(&mut self, mask: ThreadFlags) -> Option<ThreadFlags> {
        let thread = self.current().unwrap();
        if thread.flags & mask != 0 {
            let res = thread.flags & mask;
            thread.flags &= !res;
            Some(res)
        } else {
            let thread_id = thread.pid;
            self.set_state(thread_id, ThreadState::FlagBlocked(WaitMode::Any(mask)));
            crate::schedule();
            None
        }
    }

    fn flag_wait_one(&mut self, mask: ThreadFlags) -> Option<ThreadFlags> {
        let thread = self.current().unwrap();
        if thread.flags & mask != 0 {
            let mut res = thread.flags & mask;
            // clear all but least significant bit
            res &= !res + 1;
            thread.flags &= !res;
            Some(res)
        } else {
            let thread_id = thread.pid;
            self.set_state(thread_id, ThreadState::FlagBlocked(WaitMode::Any(mask)));
            crate::schedule();
            None
        }
    }
}
