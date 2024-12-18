use critical_section::CriticalSection;

use crate::{thread::Thread, RunqueueId, ThreadId, ThreadState, SCHEDULER};

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
        SCHEDULER.with_mut_cs(cs, |mut scheduler| {
            let &mut Thread { pid, prio, .. } = scheduler
                .current()
                .expect("Function should be called inside a thread context.");
            let mut curr = None;
            let mut next = self.head;
            while let Some(n) = next {
                if scheduler.get_unchecked_mut(n).prio < prio {
                    break;
                }
                curr = next;
                next = scheduler.thread_blocklist[usize::from(n)];
            }
            scheduler.thread_blocklist[usize::from(pid)] = next;
            let inherit_priority = match curr {
                Some(curr) => {
                    scheduler.thread_blocklist[usize::from(curr)] = Some(pid);
                    None
                }
                None => {
                    self.head = Some(pid);
                    Some(prio)
                }
            };
            scheduler.set_state(pid, state);
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
        SCHEDULER.with_mut_cs(cs, |mut scheduler| {
            self.head = scheduler.thread_blocklist[usize::from(head)].take();
            let old_state = scheduler.set_state(head, ThreadState::Running);
            Some((head, old_state))
        })
    }

    /// Determines if this [`ThreadList`] is empty.
    pub fn is_empty(&self, _cs: CriticalSection) -> bool {
        self.head.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_type_sizes() {
        assert_eq!(size_of::<ThreadId>(), 1);
        assert_eq!(size_of::<ThreadList>(), 2);
    }
}
