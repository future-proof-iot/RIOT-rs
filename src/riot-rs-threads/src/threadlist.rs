use critical_section::CriticalSection;

use crate::{thread::Thread, RunqueueId, ThreadId, ThreadState, THREADS};

/// Manages blocked [`super::Thread`]s for a resource, and triggering the scheduler when needed.
#[derive(Debug, Default)]
pub struct ThreadList {
    /// Next thread to run once the resource is available.
    head: Option<ThreadId>,
}

impl ThreadList {
    /// Creates a new empty [`ThreadList`]
    pub const fn new() -> Self {
        Self { head: None }
    }

    /// Puts the current (blocked) thread into this [`ThreadList`] and triggers the scheduler.
    ///
    /// Returns a `RunqueueId` if the highest priority among the waiters in the list has changed.
    ///
    /// # Panics
    ///
    /// Panics if this is called outside of a thread context.
    pub fn put_current(&mut self, cs: CriticalSection, state: ThreadState) -> Option<RunqueueId> {
        THREADS.with_mut_cs(cs, |mut threads| {
            let &mut Thread { pid, prio, .. } = threads
                .current()
                .expect("Function should be called inside a thread context.");
            let mut curr = None;
            let mut next = self.head;
            while let Some(n) = next {
                if threads.get_unchecked_mut(n).prio < prio {
                    break;
                }
                curr = next;
                next = threads.thread_blocklist[usize::from(n)];
            }
            threads.thread_blocklist[usize::from(pid)] = next;
            let inherit_priority = match curr {
                Some(curr) => {
                    threads.thread_blocklist[usize::from(curr)] = Some(pid);
                    None
                }
                _ => {
                    self.head = Some(pid);
                    Some(prio)
                }
            };
            threads.set_state(pid, state);
            crate::schedule();
            inherit_priority
        })
    }

    /// Removes the head from this [`ThreadList`].
    ///
    /// Sets the thread's [`ThreadState`] to [`ThreadState::Running`] and triggers
    /// the scheduler.
    ///
    /// Returns the thread's [`ThreadId`] and its previous [`ThreadState`].
    pub fn pop(&mut self, cs: CriticalSection) -> Option<(ThreadId, ThreadState)> {
        let head = self.head?;
        THREADS.with_mut_cs(cs, |mut threads| {
            self.head = threads.thread_blocklist[usize::from(head)].take();
            let old_state = threads.set_state(head, ThreadState::Running);
            crate::schedule();
            Some((head, old_state))
        })
    }

    /// Determines if this [`ThreadList`] is empty.
    pub fn is_empty(&self, _cs: CriticalSection) -> bool {
        self.head.is_none()
    }
}
