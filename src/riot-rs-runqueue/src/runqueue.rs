// Disable indexing lints for now
#![allow(clippy::indexing_slicing)]

use core::mem;

use self::clist::CList;

const USIZE_BITS: usize = mem::size_of::<usize>() * 8;

/// Runqueue number.
pub type RunqueueId = u8;
pub type ThreadId = u8;
pub type CoreId = u8;

trait FromBitmap: Sized {
    fn from_bitmap(bitmap: usize) -> Option<Self>;
}
impl FromBitmap for u8 {
    fn from_bitmap(bitmap: usize) -> Option<Self> {
        if bitmap == 0 {
            return None;
        }
        Some(ffs(bitmap) as ThreadId - 1)
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
/// an `[RunqueueId; N_QUEUES]` array for the list tail indexes
/// and an `[ThreadId; N_THREADS]` for the list next indexes.
pub struct RunQueue<const N_QUEUES: usize, const N_THREADS: usize, const N_CORES: usize = 1> {
    /// Bitcache that represents the currently used queues
    /// in `0..N_QUEUES`.
    bitcache: usize,
    queues: clist::CList<N_QUEUES, N_THREADS>,
    next: [Option<ThreadId>; N_CORES],
}

impl<const N_QUEUES: usize, const N_THREADS: usize, const N_CORES: usize>
    RunQueue<N_QUEUES, N_THREADS, N_CORES>
{
    // NOTE: we don't impl Default here because hax does not support it yet. When it does, we
    // should impl it.
    #[allow(clippy::new_without_default)]
    pub const fn new() -> RunQueue<N_QUEUES, N_THREADS, N_CORES> {
        // unfortunately we cannot assert!() on N_QUEUES and N_THREADS,
        // as panics in const fn's are not (yet) implemented.
        RunQueue {
            bitcache: 0,
            queues: CList::new(),
            next: [None; N_CORES],
        }
    }

    /// Adds thread with pid `n` to runqueue number `rq`.
    ///
    /// Returns a [`CoreId`] if the allocation for this core changed.
    ///
    pub fn add(&mut self, n: ThreadId, rq: RunqueueId) -> Option<CoreId> {
        debug_assert!((n as usize) < N_THREADS);
        debug_assert!((rq as usize) < N_QUEUES);
        self.bitcache |= 1 << rq;
        self.queues.push(n, rq);
        self.reallocate()
    }

    /// Removes thread with pid `n` from runqueue number `rq`.
    ///
    /// Returns a [`CoreId`] if the allocation for this core changed.
    ///
    /// # Panics
    ///
    /// Panics if `n` is not the queue's head.
    /// This is fine, RIOT-rs only ever calls `del()` for the current thread.
    pub fn del(&mut self, n: ThreadId, rq: RunqueueId) -> Option<CoreId> {
        debug_assert!((n as usize) < N_THREADS);
        debug_assert!((rq as usize) < N_QUEUES);

        if N_CORES == 1 {
            let popped = self.queues.pop_head(rq);
            assert_eq!(popped, Some(n));
        } else {
            if self.queues.peek_head(rq) == Some(n) {
                let popped = self.queues.pop_head(rq);
                assert_eq!(popped, Some(n));
            } else {
                self.queues.del(n, rq);
            }
        }

        if self.queues.is_empty(rq) {
            self.bitcache &= !(1 << rq);
        }
        self.reallocate()
    }

    /// Returns the next thread that should run on this core.
    pub fn get_next_for_core(&self, core: CoreId) -> Option<ThreadId> {
        if core as usize >= N_CORES {
            return None;
        }
        self.next[core as usize]
    }

    /// Advances from thread `n` in runqueue number `rq`.
    ///
    /// This is used to "yield" to another thread of *the same* priority.
    /// Compared to [`RunQueue::advance`], this method allows to advance from a thread that
    /// is not necessarily the head of the runqueue.
    ///
    /// Returns a [`CoreId`] if the allocation for this core changed.
    ///
    /// **Warning: This changes the order of the runqueue because the thread is moved to the
    /// tail of the queue.**
    pub fn advance_from(&mut self, n: ThreadId, rq: RunqueueId) -> Option<CoreId> {
        debug_assert!((rq as usize) < N_QUEUES);
        if Some(n) == self.queues.peek_head(rq) {
            self.queues.advance(rq);
        } else {
            // If the thread is not the head remove it
            // from queue and re-insert it at tail.
            self.queues.del(n, rq);
            self.queues.push(n, rq);
        }
        self.reallocate()
    }

    /// Update `self.next` so that the highest `N_CORES` threads
    /// are allocated.
    ///
    /// This only changes allocations if a thread was previously allocated
    /// and is now not part of the new list anymore, or the other way around.
    /// It assumes that there was maximum one change in the runqueue since the
    /// last reallocation (only one add/ delete or a runqueue advancement)!
    ///
    /// Returns a [`CoreId`] if the allocation for this core changed.
    ///
    /// The complexity of this call is O(n).
    fn reallocate(&mut self) -> Option<CoreId> {
        if N_CORES == 1 {
            let next = self.peek_head(self.bitcache);
            if next == self.next[0] {
                return None;
            }
            self.next[0] = next;
            return Some(0);
        }
        let next = self.get_next_n();
        let mut bitmap_next = 0;
        let mut bitmap_allocated = 0;
        for i in 0..N_CORES {
            if let Some(id) = next[i] {
                bitmap_next |= 1 << id
            }
            if let Some(id) = self.next[i] {
                bitmap_allocated |= 1 << id
            }
        }
        if bitmap_next == bitmap_allocated {
            return None;
        }
        let diff = bitmap_next ^ bitmap_allocated;
        let prev_allocated = ThreadId::from_bitmap(bitmap_allocated & diff);
        let new_allocated = ThreadId::from_bitmap(bitmap_next & diff);

        let changed_core = self.next.iter().position(|i| *i == prev_allocated).unwrap();
        self.next[changed_core] = new_allocated;
        return Some(changed_core as CoreId);
    }

    /// Returns the `n` highest priority threads in the [`Runqueue`].
    ///
    /// This iterates through all non-empty runqueues with descending
    /// priority, until `N_CORES` threads have been found or all
    /// queues have been checked.
    ///
    /// Complexity is O(n).
    pub fn get_next_n(&self) -> [Option<ThreadId>; N_CORES] {
        let mut next_list = [None; N_CORES];
        let mut bitcache = self.bitcache;
        // Get head from highest priority queue.
        let mut head = match self.peek_head(bitcache) {
            Some(head) => {
                next_list[0] = Some(head);
                head
            }
            None => return next_list,
        };
        let mut thread = head;
        // Iterate through threads in the queue.
        for i in 1..N_CORES {
            thread = self.queues.peek_next(thread);
            if thread == head {
                // Switch to next runqueue.
                bitcache &= !(1 << (ffs(bitcache) - 1));
                head = match self.peek_head(bitcache) {
                    Some(h) => h,
                    // Early return instead of break, to make hax happy.
                    None => return next_list,
                };
                thread = head;
            };
            next_list[i] = Some(thread);
        }
        next_list
    }

    fn peek_head(&self, bitcache: usize) -> Option<ThreadId> {
        // Switch to highest priority runqueue remaining
        // in the bitcache.
        let rq = match RunqueueId::from_bitmap(bitcache) {
            Some(rq) => rq,
            None => return None,
        };
        self.queues.peek_head(rq)
    }
}

impl<const N_QUEUES: usize, const N_THREADS: usize> RunQueue<N_QUEUES, N_THREADS> {
    /// Returns the pid that should run next.
    ///
    /// Returns the next runnable thread of
    /// the runqueue with the highest index.
    pub fn get_next(&self) -> Option<ThreadId> {
        self.get_next_for_core(0)
    }

    /// Advances runqueue number `rq`.
    ///
    /// This is used to "yield" to another thread of *the same* priority.
    ///
    /// Returns a [`CoreId`] if the allocation for this core changed.
    pub fn advance(&mut self, rq: RunqueueId) -> Option<ThreadId> {
        debug_assert!((rq as usize) < N_QUEUES);
        self.queues.advance(rq);
        self.reallocate()
    }
}

fn ffs(val: usize) -> u32 {
    USIZE_BITS as u32 - val.leading_zeros()
}

mod clist {
    //! This module implements an array of `N_QUEUES` circular linked lists over an
    //! array of size `N_THREADS`.
    //! The array is used for "next" pointers, so each integer value in the array
    //! corresponds to one element, which can only be in one of the lists.
    use super::{RunqueueId, ThreadId};

    #[derive(Debug, Copy, Clone)]
    pub struct CList<const N_QUEUES: usize, const N_THREADS: usize> {
        tail: [RunqueueId; N_QUEUES],
        next_idxs: [ThreadId; N_THREADS],
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

        pub fn is_empty(&self, rq: RunqueueId) -> bool {
            self.tail[rq as usize] == Self::sentinel()
        }

        pub fn push(&mut self, n: ThreadId, rq: RunqueueId) {
            assert!(n < Self::sentinel());
            if self.next_idxs[n as usize] == Self::sentinel() {
                if self.tail[rq as usize] == Self::sentinel() {
                    // rq is empty, link both tail and n.next to n
                    self.tail[rq as usize] = n;
                    self.next_idxs[n as usize] = n;
                } else {
                    // rq has an entry already, so
                    // 1. n.next = old_tail.next ("first" in list)
                    self.next_idxs[n as usize] = self.next_idxs[self.tail[rq as usize] as usize];
                    // 2. old_tail.next = n
                    self.next_idxs[self.tail[rq as usize] as usize] = n;
                    // 3. tail = n
                    self.tail[rq as usize] = n;
                }
            }
        }

        /// Delete a thread from the runqueue.
        pub fn del(&mut self, n: ThreadId, rq: RunqueueId) {
            if self.next_idxs[n as usize] == Self::sentinel() {
                // Thread is not in rq, do nothing.
                return;
            }

            if self.next_idxs[n as usize] == n {
                // `n` should always be the tail in this case, but better be
                // safe and double-check.
                if self.tail[rq as usize] == n {
                    // `n` bites itself, so there's only one entry.
                    // Clear tail.
                    self.tail[rq as usize] = Self::sentinel();
                }
            } else {
                let next = self.next_idxs[n as usize];

                // Find previous in list and update its next-idx.
                let prev = self
                    .next_idxs
                    .iter()
                    .position(|next_idx| *next_idx == n)
                    .expect("List is circular.");
                self.next_idxs[prev] = next as ThreadId;

                // Update tail if the thread was the tail.
                if self.tail[rq as usize] == n {
                    self.tail[rq as usize] = prev as ThreadId;
                }
            }

            // Clear thread's value.
            self.next_idxs[n as usize] = Self::sentinel();
        }

        pub fn pop_head(&mut self, rq: RunqueueId) -> Option<ThreadId> {
            if self.tail[rq as usize] == Self::sentinel() {
                // rq is empty, do nothing
                None
            } else {
                let head = self.next_idxs[self.tail[rq as usize] as usize];
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
        }

        pub fn peek_head(&self, rq: RunqueueId) -> Option<ThreadId> {
            if self.tail[rq as usize] == Self::sentinel() {
                None
            } else {
                Some(self.next_idxs[self.tail[rq as usize] as usize])
            }
        }

        pub fn advance(&mut self, rq: RunqueueId) {
            if self.tail[rq as usize] != Self::sentinel() {
                self.tail[rq as usize] = self.next_idxs[self.tail[rq as usize] as usize];
            }
        }

        pub fn peek_next(&self, curr: ThreadId) -> ThreadId {
            self.next_idxs[curr as usize]
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
                clist.push(i as ThreadId, 0);
            }
            for i in 0..(N - 1) {
                println!("{}", i);
                assert_eq!(clist.pop_head(0), Some(i as ThreadId));
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
