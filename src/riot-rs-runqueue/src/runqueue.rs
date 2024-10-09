// Disable indexing lints for now
#![allow(clippy::indexing_slicing)]

use core::mem;

use self::clist::CList;

const USIZE_BITS: usize = mem::size_of::<usize>() * 8;

/// Runqueue number.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RunqueueId(u8);

impl RunqueueId {
    pub const fn new(value: u8) -> Self {
        Self(value)
    }
}

impl From<RunqueueId> for usize {
    fn from(value: RunqueueId) -> Self {
        usize::from(value.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ThreadId(u8);

impl ThreadId {
    pub const fn new(value: u8) -> Self {
        Self(value)
    }
}

impl From<ThreadId> for usize {
    fn from(value: ThreadId) -> Self {
        usize::from(value.0)
    }
}

/// Runqueue for `N_QUEUES`, supporting `N_THREADS` total.
///
/// Assumptions:
/// - runqueue numbers (corresponding priorities) are 0..N_QUEUES (exclusive)
/// - higher runqueue number ([`RunqueueId`]) means higher priority
/// - runqueue numbers fit in usize bits (supporting max 32 priority levels)
/// - [`ThreadId`]s range from 0..N_THREADS
/// - `N_THREADS` is <255 (as u8 is used to store them, but 0xFF is used as
///   special value)
///
/// The current implementation needs an usize for the bit cache,
/// an `[u8; N_QUEUES]` array for the list tail indexes
/// and an `[u8; N_THREADS]` for the list next indexes.
#[derive(Default)]
pub struct RunQueue<const N_QUEUES: usize, const N_THREADS: usize> {
    /// Bitcache that represents the currently used queues
    /// in `0..N_QUEUES`.
    bitcache: usize,
    queues: clist::CList<N_QUEUES, N_THREADS>,
}

impl<const N_QUEUES: usize, const N_THREADS: usize> RunQueue<{ N_QUEUES }, { N_THREADS }> {
    pub const fn new() -> RunQueue<{ N_QUEUES }, { N_THREADS }> {
        // unfortunately we cannot assert!() on N_QUEUES and N_THREADS,
        // as panics in const fn's are not (yet) implemented.
        RunQueue {
            bitcache: 0,
            queues: CList::new(),
        }
    }

    /// Adds thread with pid `n` to runqueue number `rq`.
    pub fn add(&mut self, n: ThreadId, rq: RunqueueId) {
        debug_assert!(usize::from(n) < N_THREADS);
        debug_assert!(usize::from(rq) < N_QUEUES);
        self.bitcache |= 1 << rq.0;
        self.queues.push(n.0, rq.0);
    }

    /// Returns the head of the runqueue without removing it.
    pub fn peek_head(&self, rq: RunqueueId) -> Option<ThreadId> {
        self.queues.peek_head(rq.0).map(ThreadId::new)
    }

    /// Removes thread with pid `n` from runqueue number `rq`.
    ///
    /// # Panics
    ///
    /// Panics if `n` is not the queue's head.
    /// This is fine, RIOT-rs only ever calls `pop_head()` for the current thread.
    pub fn pop_head(&mut self, n: ThreadId, rq: RunqueueId) {
        debug_assert!(usize::from(n) < N_THREADS);
        debug_assert!(usize::from(rq) < N_QUEUES);
        let popped = self.queues.pop_head(rq.0);
        //
        assert_eq!(popped, Some(n.0));
        if self.queues.is_empty(rq.0) {
            self.bitcache &= !(1 << rq.0);
        }
    }

    /// Removes thread with pid `n`.
    pub fn del(&mut self, n: ThreadId) {
        if let Some(empty_runqueue) = self.queues.del(n.0) {
            self.bitcache &= !(1 << empty_runqueue);
        }
    }

    fn ffs(val: usize) -> u32 {
        USIZE_BITS as u32 - val.leading_zeros()
    }

    /// Returns the pid that should run next.
    ///
    /// Returns the next runnable thread of
    /// the runqueue with the highest index.
    pub fn get_next(&self) -> Option<ThreadId> {
        let rq_ffs = Self::ffs(self.bitcache);
        if rq_ffs == 0 {
            return None;
        }
        let rq = rq_ffs as u8 - 1;
        self.queues.peek_head(rq).map(ThreadId::new)
    }

    /// Advances runqueue number `rq`.
    ///
    /// This is used to "yield" to another thread of *the same* priority.
    ///
    /// Returns `false` if the operation had no effect, i.e. when the runqueue
    /// is empty or only contains a single thread.
    pub fn advance(&mut self, rq: RunqueueId) -> bool {
        debug_assert!((usize::from(rq)) < N_QUEUES);
        self.queues.advance(rq.0)
    }
}

mod clist {
    //! This module implements an array of `N_QUEUES` circular linked lists over an
    //! array of size `N_THREADS`.
    //!
    //! The array is used for "next" pointers, so each integer value in the array
    //! corresponds to one element, which can only be in one of the lists.
    #[derive(Debug, Copy, Clone)]
    pub struct CList<const N_QUEUES: usize, const N_THREADS: usize> {
        tail: [u8; N_QUEUES],
        next_idxs: [u8; N_THREADS],
    }

    impl<const N_QUEUES: usize, const N_THREADS: usize> Default for CList<N_QUEUES, N_THREADS> {
        fn default() -> Self {
            Self::new()
        }
    }

    impl<const N_QUEUES: usize, const N_THREADS: usize> CList<N_QUEUES, N_THREADS> {
        pub const fn new() -> Self {
            // TODO: ensure N fits in u8
            // assert!(N<255); is not allowed in const because it could panic
            CList {
                tail: [Self::sentinel(); N_QUEUES],
                next_idxs: [Self::sentinel(); N_THREADS],
            }
        }

        pub const fn sentinel() -> u8 {
            0xFF
        }

        pub fn is_empty(&self, rq: u8) -> bool {
            self.tail[rq as usize] == Self::sentinel()
        }

        pub fn push(&mut self, n: u8, rq: u8) {
            assert!(n < Self::sentinel());
            if self.next_idxs[n as usize] != Self::sentinel() {
                return;
            }

            if let Some(head) = self.peek_head(rq) {
                // rq has an entry already, so
                // 1. n.next = old_tail.next ("first" in list)
                self.next_idxs[n as usize] = head;
                // 2. old_tail.next = n
                self.next_idxs[self.tail[rq as usize] as usize] = n;
                // 3. tail = n
                self.tail[rq as usize] = n;
            } else {
                // rq is empty, link both tail and n.next to n
                self.tail[rq as usize] = n;
                self.next_idxs[n as usize] = n;
            }
        }

        /// Removes a thread from the list.
        ///
        /// If the thread was the only thread in its runqueue, `Some` is returned
        /// with the ID of the now empty runqueue.
        pub fn del(&mut self, n: u8) -> Option<u8> {
            if self.next_idxs[n as usize] == Self::sentinel() {
                return None;
            }
            let mut empty_runqueue = None;

            // Find previous thread in circular runqueue.
            let prev = position(&self.next_idxs, n)?;

            // Handle if thread is tail of a runqueue.
            if let Some(rq) = position(&self.tail, n) {
                if prev == n as usize {
                    // Runqueue is empty now.
                    self.tail[rq] = Self::sentinel();
                    empty_runqueue = Some(rq as u8);
                } else {
                    self.tail[rq] = prev as u8;
                }
            }
            self.next_idxs[prev] = self.next_idxs[n as usize];
            self.next_idxs[n as usize] = Self::sentinel();
            empty_runqueue
        }

        pub fn pop_head(&mut self, rq: u8) -> Option<u8> {
            let head = self.peek_head(rq)?;

            if head == self.tail[rq as usize] {
                // rq's tail bites itself, so there's only one entry.
                // so, clear tail.
                self.tail[rq as usize] = Self::sentinel();
                // rq is now empty
            } else {
                // rq has multiple entries,
                // so set tail.next to head.next (second in list)
                self.next_idxs[self.tail[rq as usize] as usize] = self.next_idxs[head as usize];
            }

            // now clear head's next value
            self.next_idxs[head as usize] = Self::sentinel();
            Some(head)
        }

        #[inline]
        pub fn peek_head(&self, rq: u8) -> Option<u8> {
            if self.is_empty(rq) {
                None
            } else {
                Some(self.next_idxs[self.tail[rq as usize] as usize])
            }
        }

        pub fn advance(&mut self, rq: u8) -> bool {
            let tail = self.tail[rq as usize];
            let head = self.next_idxs[tail as usize];
            if tail == head {
                // Catches the case that the runqueue only has a single element,
                // or is empty (in which case head == tail == Self::sentinel())
                return false;
            }
            self.tail[rq as usize] = head;
            true
        }
    }

    /// Helper function that is needed because hax doesn't support `Iterator::position` yet.
    fn position<const N: usize>(slice: &[u8; N], search_item: u8) -> Option<usize> {
        let mut i = 0;
        while i < N && slice[i] != search_item {
            i += 1;
        }
        if i < N {
            Some(i)
        } else {
            None
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_clist_basic() {
            let mut clist: CList<8, 32> = CList::new();
            assert!(clist.is_empty(0));
            clist.push(0, 0);
            assert_eq!(clist.pop_head(0), Some(0));
            assert_eq!(clist.pop_head(0), None);
        }

        #[test]
        fn test_clist_push_already_in_list() {
            let mut clist: CList<8, 32> = CList::new();
            assert!(clist.is_empty(0));
            clist.push(0, 0);
            clist.push(0, 0);
            assert_eq!(clist.pop_head(0), Some(0));
            assert_eq!(clist.pop_head(0), None);
            assert!(clist.is_empty(0));
        }

        #[test]
        fn test_clist_push_two() {
            let mut clist: CList<8, 32> = CList::new();
            assert!(clist.is_empty(0));
            clist.push(0, 0);
            clist.push(1, 0);
            assert_eq!(clist.pop_head(0), Some(0));
            assert_eq!(clist.pop_head(0), Some(1));
            assert_eq!(clist.pop_head(0), None);
            assert!(clist.is_empty(0));
        }

        #[test]
        fn test_clist_push_all() {
            const N: usize = 255;
            let mut clist: CList<8, N> = CList::new();
            assert!(clist.is_empty(0));
            for i in 0..(N - 1) {
                println!("pushing {}", i);
                clist.push(i as u8, 0);
            }
            for i in 0..(N - 1) {
                println!("{}", i);
                assert_eq!(clist.pop_head(0), Some(i as u8));
            }
            assert_eq!(clist.pop_head(0), None);
            assert!(clist.is_empty(0));
        }

        #[test]
        fn test_clist_advance() {
            let mut clist: CList<8, 32> = CList::new();
            assert!(clist.is_empty(0));
            clist.push(0, 0);
            clist.push(1, 0);
            clist.advance(0);
            assert_eq!(clist.pop_head(0), Some(1));
            assert_eq!(clist.pop_head(0), Some(0));
            assert_eq!(clist.pop_head(0), None);
            assert!(clist.is_empty(0));
        }

        #[test]
        fn test_clist_peek_head() {
            let mut clist: CList<8, 32> = CList::new();
            assert!(clist.is_empty(0));
            clist.push(0, 0);
            clist.push(1, 0);
            assert_eq!(clist.peek_head(0), Some(0));
            assert_eq!(clist.peek_head(0), Some(0));
            assert_eq!(clist.pop_head(0), Some(0));
            assert_eq!(clist.peek_head(0), Some(1));
            assert_eq!(clist.pop_head(0), Some(1));
            assert_eq!(clist.peek_head(0), None);
            assert_eq!(clist.peek_head(0), None);
            assert_eq!(clist.pop_head(0), None);
            assert!(clist.is_empty(0));
        }
    }
}
