use critical_section::CriticalSection;

use crate::{arch, ThreadId, ThreadState, THREADS};

/// Manages blocked [`super::Thread`]s for a resource, and triggering the scheduler when needed.
#[derive(Debug)]
pub struct ThreadList {
    /// Next thread to run once the resource is available.
    pub head: Option<ThreadId>,
}

impl ThreadList {
    /// Creates a new empty [`ThreadList`]
    pub const fn new() -> Self {
        Self { head: None }
    }

    /// Puts the current (blocked) thread into this [`ThreadList`] and triggers the scheduler.
    pub fn put_current(&mut self, cs: CriticalSection, state: ThreadState) {
        THREADS.with_mut_cs(cs, |mut threads| {
            let thread_id = threads.current_thread.unwrap();
            threads.thread_blocklist[thread_id as usize] = self.head;
            self.head = Some(thread_id);
            threads.set_state(thread_id, state);
            arch::schedule();
        });
    }

    /// Removes the head from this [`ThreadList`].
    ///
    /// Sets the thread's [`ThreadState`] to [`ThreadState::Running`] and triggers
    /// the scheduler.
    ///
    /// Returns the thread's [`ThreadId`] and it's previous [`ThreadState`].
    pub fn pop(&mut self, cs: CriticalSection) -> Option<(ThreadId, ThreadState)> {
        if let Some(head) = self.head {
            let old_state = THREADS.with_mut_cs(cs, |mut threads| {
                self.head = threads.thread_blocklist[head as usize].take();
                let old_state = threads.set_state(head, ThreadState::Running);
                arch::schedule();
                old_state
            });
            Some((head, old_state))
        } else {
            None
        }
    }

    /// Determines if this [`ThreadList`] is empty
    pub fn is_empty(&self, _cs: CriticalSection) -> bool {
        self.head.is_none()
    }
}
