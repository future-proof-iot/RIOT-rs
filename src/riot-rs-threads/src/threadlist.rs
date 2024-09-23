use critical_section::CriticalSection;

use crate::{thread::Thread, ThreadId, ThreadState, THREADS};

/// Manages blocked [`super::Thread`]s for a resource, and triggering the scheduler when needed.
#[derive(Debug, Default)]
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
    ///
    /// # Panics
    ///
    /// Panics if this is called outside of a thread context.
    pub fn put_current(&mut self, cs: CriticalSection, state: ThreadState) {
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
            match curr {
                Some(curr) => threads.thread_blocklist[usize::from(curr)] = Some(pid),
                _ => self.head = Some(pid),
            }
            threads.set_state(pid, state);
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
        if let Some(head) = self.head {
            let old_state = THREADS.with_mut_cs(cs, |mut threads| {
                self.head = threads.thread_blocklist[usize::from(head)].take();
                let old_state = threads.set_state(head, ThreadState::Running);
                crate::schedule();
                old_state
            });
            Some((head, old_state))
        } else {
            None
        }
    }

    /// Determines if this [`ThreadList`] is empty.
    pub fn is_empty(&self, _cs: CriticalSection) -> bool {
        self.head.is_none()
    }
}
