use core::mem;
use core::sync::atomic::compiler_fence;
use core::sync::atomic::Ordering;

use self::clist::CList;

const USIZE_BITS: usize = mem::size_of::<usize>() * 8;

pub type RunqueueId = u8;
pub type ThreadId = u8;

/// Runqueue for N_QUEUES, supporting N_THREADS total.
///
/// assumptions:
/// - higher runqueue number means higher priority
/// - runqueue numbers (corresponding priorities) are 0..N_QUEUES (exclusive)
/// - runqueue numbers fit in usize bits (supporting max 32 priority levels)
/// - pids range from 0..N_THREADS
/// - N_THREADS is <255 (as u8 is used to store them, but 0xFF is used as
///   special value)
///
///   The current implementation needs an usize for the bit cache,
///   an [u8; N_QUEUES] array for the list tail indexes
///   and an [u8; N_THREADS] for the list next indexes.
pub struct RunQueue<const N_QUEUES: usize, const N_THREADS: usize> {
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

    /// add thread with pid n to runqueue number rq
    pub fn add(&mut self, n: ThreadId, rq: RunqueueId) {
        debug_assert!((n as usize) < N_THREADS);
        debug_assert!((rq as usize) < N_QUEUES);
        self.bitcache |= 1 << rq;
        self.queues.push(n as u8, rq as u8);
    }

    /// remove thread with pid n from runqueue number rq
    /// @note: this implementation fails if "n" is not the queue's head.
    /// This is fine, RIOT-rs only ever calls del() for the current thread.
    pub fn del(&mut self, n: ThreadId, rq: RunqueueId) {
        debug_assert!((n as usize) < N_THREADS);
        debug_assert!((rq as usize) < N_QUEUES);
        let popped = self.queues.pop_head(rq as u8);
        //
        assert_eq!(popped, Some(n as u8));
        if self.queues.empty(rq) {
            self.bitcache &= !(1 << rq);
        }
    }

    fn ffs(val: usize) -> u32 {
        USIZE_BITS as u32 - val.leading_zeros()
    }

    /// get pid that should run next
    /// returns the next runnable thread of
    /// the runqueue with the highest index
    pub fn get_next(&self) -> Option<u8> {
        compiler_fence(Ordering::AcqRel);
        let rq_ffs = Self::ffs(self.bitcache);
        if rq_ffs > 0 {
            let rq = (rq_ffs - 1) as usize;
            self.queues.peek_head(rq as u8)
        } else {
            None
        }
    }

    /// advance runqueue number rq
    /// (this is used to "yield" to another thread of *the same* priority)
    pub fn advance(&mut self, rq: RunqueueId) {
        debug_assert!((rq as usize) < N_QUEUES);
        self.queues.advance(rq as u8)
    }
}

mod clist {
    use super::{RunqueueId, ThreadId};

    #[derive(Debug, Copy, Clone)]
    pub struct CList<const N_QUEUES: usize, const N_THREADS: usize> {
        tail: [u8; N_QUEUES],
        next_idxs: [u8; N_THREADS],
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

        pub fn empty(&self, rq: RunqueueId) -> bool {
            self.tail[rq as usize] == Self::sentinel()
        }

        pub fn push(&mut self, n: ThreadId, rq: RunqueueId) {
            assert!(n < Self::sentinel());
            if self.next_idxs[n as usize] == Self::sentinel() {
                if self.tail[rq as usize] == Self::sentinel() {
                    self.tail[rq as usize] = n;
                    self.next_idxs[n as usize] = n;
                } else {
                    self.next_idxs[n as usize] = self.next_idxs[self.tail[rq as usize] as usize];
                    self.next_idxs[self.tail[rq as usize] as usize] = n;
                    self.tail[rq as usize] = n;
                }
            }
        }

        pub fn pop_head(&mut self, rq: RunqueueId) -> Option<u8> {
            if self.tail[rq as usize] == Self::sentinel() {
                None
            } else {
                let res = self.next_idxs[self.tail[rq as usize] as usize];
                if res == self.tail[rq as usize] {
                    self.tail[rq as usize] = Self::sentinel();
                } else {
                    self.next_idxs[self.tail[rq as usize] as usize] = self.next_idxs[res as usize];
                }
                Some(res)
            }
        }

        pub fn peek_head(&self, rq: RunqueueId) -> Option<u8> {
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
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_clist_basic() {
            let mut clist: CList<8, 32> = CList::new();
            assert!(clist.empty(0));
            clist.push(0, 0);
            assert_eq!(clist.pop_head(0), Some(0));
            assert_eq!(clist.pop_head(0), None);
        }

        #[test]
        fn test_clist_push_already_in_list() {
            let mut clist: CList<8, 32> = CList::new();
            assert!(clist.empty(0));
            clist.push(0, 0);
            clist.push(0, 0);
            assert_eq!(clist.pop_head(0), Some(0));
            assert_eq!(clist.pop_head(0), None);
            assert!(clist.empty(0));
        }

        #[test]
        fn test_clist_push_two() {
            let mut clist: CList<8, 32> = CList::new();
            assert!(clist.empty(0));
            clist.push(0, 0);
            clist.push(1, 0);
            assert_eq!(clist.pop_head(0), Some(0));
            assert_eq!(clist.pop_head(0), Some(1));
            assert_eq!(clist.pop_head(0), None);
            assert!(clist.empty(0));
        }

        #[test]
        fn test_clist_push_all() {
            const N: usize = 255;
            let mut clist: CList<8, N> = CList::new();
            assert!(clist.empty(0));
            for i in 0..(N - 1) {
                println!("pushing {}", i);
                clist.push(i as u8, 0);
            }
            for i in 0..(N - 1) {
                println!("{}", i);
                assert_eq!(clist.pop_head(0), Some(i as u8));
            }
            assert_eq!(clist.pop_head(0), None);
            assert!(clist.empty(0));
        }

        #[test]
        fn test_clist_advance() {
            let mut clist: CList<8, 32> = CList::new();
            assert!(clist.empty(0));
            clist.push(0, 0);
            clist.push(1, 0);
            clist.advance(0);
            assert_eq!(clist.pop_head(0), Some(1));
            assert_eq!(clist.pop_head(0), Some(0));
            assert_eq!(clist.pop_head(0), None);
            assert!(clist.empty(0));
        }

        #[test]
        fn test_clist_peek_head() {
            let mut clist: CList<8, 32> = CList::new();
            assert!(clist.empty(0));
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
            assert!(clist.empty(0));
        }
    }
}
