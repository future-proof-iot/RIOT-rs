use critical_section::CriticalSection;

use crate::{ThreadId, ThreadState, THREADS};

/// Manages blocked [`super::Thread`]s for a resource, and triggering the scheduler when needed.
#[derive(Debug, Default)]
pub struct ThreadList {
    /// Last thread that was added to the list.
    ///
    /// `Thread.thread_blocklist` is circular, therefore the next thread for
    /// the tail is the head of the list.
    pub tail: Option<ThreadId>,
}

impl ThreadList {
    /// Creates a new empty [`ThreadList`]
    pub const fn new() -> Self {
        Self { tail: None }
    }

    /// Puts the current (blocked) thread into this [`ThreadList`] and triggers the scheduler.
    pub fn put_current(&mut self, cs: CriticalSection, state: ThreadState) {
        THREADS.with_mut_cs(cs, |mut threads| {
            let thread_id = threads.current_pid().unwrap();
            if let Some(tail) = self.tail {
                let head = threads.thread_blocklist[usize::from(tail)];
                threads.thread_blocklist[usize::from(thread_id)] = head;
                threads.thread_blocklist[usize::from(tail)] = Some(thread_id);
            } else {
                threads.thread_blocklist[usize::from(thread_id)] = Some(thread_id);
            }
            self.tail = Some(thread_id);
            threads.set_state(thread_id, state);
            crate::schedule();
        });
    }

    /// Removes the head from this [`ThreadList`].
    ///
    /// Sets the thread's [`ThreadState`] to [`ThreadState::Running`] and triggers
    /// the scheduler.
    ///
    /// Returns the thread's [`ThreadId`] and its previous [`ThreadState`].
    pub fn pop(&mut self, cs: CriticalSection) -> Option<(ThreadId, ThreadState)> {
        let tail = self.tail?;
        THREADS.with_mut_cs(cs, |mut threads| {
            let head = threads.thread_blocklist[usize::from(tail)].take()?;
            if head == tail {
                self.tail = None;
            } else {
                threads.thread_blocklist[usize::from(tail)] =
                    threads.thread_blocklist[usize::from(head)]
            }
            let old_state = threads.set_state(head, ThreadState::Running);
            crate::schedule();
            Some((head, old_state))
        })
    }

    /// Determines if this [`ThreadList`] is empty.
    pub fn is_empty(&self, _cs: CriticalSection) -> bool {
        self.tail.is_none()
    }
}
