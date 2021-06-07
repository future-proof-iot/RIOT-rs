use core::mem;
use core::sync::atomic::compiler_fence;
use core::sync::atomic::Ordering;

const USIZE_BITS: usize = mem::size_of::<usize>() * 8;

/// assumptions:
/// - runqueue numbers are 0..N (exclusive)
/// - pids are 0..USIZE_BITS (supporting max 32 threads)
/// - higher runqueue number means higher priority
use self::clist::CList;

// TODO: use atomics
pub struct RunQueue<const N_QUEUES: usize, const N_THREADS: usize> {
    bitcache: usize,
    queues: [clist::CList<N_THREADS>; N_QUEUES],
}

impl<const N_QUEUES: usize, const N_THREADS: usize> RunQueue<{ N_QUEUES }, { N_THREADS }> {
    pub const fn new() -> RunQueue<{ N_QUEUES }, { N_THREADS }> {
        RunQueue {
            bitcache: 0,
            queues: [CList::new(); N_QUEUES],
        }
    }

    /// add thread with pid n to runqueue number rq
    pub fn add(&mut self, n: usize, rq: usize) {
        debug_assert!(n < USIZE_BITS);
        debug_assert!(n < N_THREADS);
        debug_assert!(rq < N_QUEUES);
        self.bitcache |= 1 << rq;
        self.queues[rq].push(n as u8);
    }

    /// remove thread with pid n from runqueue number rq
    /// @note: this implementation fails if "n" is not the queue's head.
    /// This is fine, RIOT-rs only ever calls del() for the current thread.
    pub fn del(&mut self, n: usize, rq: usize) {
        debug_assert!(n < USIZE_BITS);
        debug_assert!(rq < N_QUEUES);
        let popped = self.queues[rq].pop_head();
        //
        assert_eq!(popped, Some(n as u8));
        if self.queues[rq].empty() {
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
            self.queues[rq].peek_head()
        } else {
            None
        }
    }

    /// advance runqueue number rq
    /// (this is used to "yield" to another thread of *the same* priority)
    pub fn advance(&mut self, rq: usize) {
        debug_assert!(rq < N_QUEUES);
        self.queues[rq].advance()
    }
}

mod clist {
    #[derive(Debug, Copy, Clone)]
    pub struct CList<const N: usize> {
        tail: u8,
        next_idxs: [u8; N],
    }

    impl<const N: usize> CList<N> {
        pub const fn new() -> Self {
            // TODO: ensure N fits in u8
            // assert!(N<255); is not allowed in const because it could panic
            CList {
                tail: Self::sentinel(),
                next_idxs: [Self::sentinel(); N],
            }
        }

        pub const fn sentinel() -> u8 {
            0xFF
        }

        pub fn empty(&self) -> bool {
            self.tail == Self::sentinel()
        }

        pub fn push(&mut self, n: u8) {
            assert!(n < Self::sentinel());
            if self.next_idxs[n as usize] == Self::sentinel() {
                if self.tail == Self::sentinel() {
                    self.tail = n;
                    self.next_idxs[n as usize] = n;
                } else {
                    self.next_idxs[n as usize] = self.next_idxs[self.tail as usize];
                    self.next_idxs[self.tail as usize] = n;
                    self.tail = n;
                }
            }
        }

        pub fn pop_head(&mut self) -> Option<u8> {
            if self.tail == Self::sentinel() {
                None
            } else {
                let res = self.next_idxs[self.tail as usize];
                if res == self.tail {
                    self.tail = Self::sentinel();
                } else {
                    self.next_idxs[self.tail as usize] = self.next_idxs[res as usize];
                }
                Some(res)
            }
        }

        pub fn peek_head(&self) -> Option<u8> {
            if self.tail == Self::sentinel() {
                None
            } else {
                Some(self.next_idxs[self.tail as usize])
            }
        }

        pub fn advance(&mut self) {
            if self.tail != Self::sentinel() {
                self.tail = self.next_idxs[self.tail as usize];
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_clist_basic() {
            let mut clist: CList<8> = CList::new();
            assert!(clist.empty());
            clist.push(0);
            assert_eq!(clist.pop_head(), Some(0));
            assert_eq!(clist.pop_head(), None);
        }

        #[test]
        fn test_clist_push_already_in_list() {
            let mut clist: CList<8> = CList::new();
            assert!(clist.empty());
            clist.push(0);
            clist.push(0);
            assert_eq!(clist.pop_head(), Some(0));
            assert_eq!(clist.pop_head(), None);
            assert!(clist.empty());
        }

        #[test]
        fn test_clist_push_two() {
            let mut clist: CList<8> = CList::new();
            assert!(clist.empty());
            clist.push(0);
            clist.push(1);
            assert_eq!(clist.pop_head(), Some(0));
            assert_eq!(clist.pop_head(), Some(1));
            assert_eq!(clist.pop_head(), None);
            assert!(clist.empty());
        }

        #[test]
        fn test_clist_push_all() {
            const N: usize = 255;
            let mut clist: CList<N> = CList::new();
            assert!(clist.empty());
            for i in 0..(N - 1) {
                println!("pushing {}", i);
                clist.push(i as u8);
            }
            for i in 0..(N - 1) {
                println!("{}", i);
                assert_eq!(clist.pop_head(), Some(i as u8));
            }
            assert_eq!(clist.pop_head(), None);
            assert!(clist.empty());
        }

        #[test]
        fn test_clist_advance() {
            let mut clist: CList<8> = CList::new();
            assert!(clist.empty());
            clist.push(0);
            clist.push(1);
            clist.advance();
            assert_eq!(clist.pop_head(), Some(1));
            assert_eq!(clist.pop_head(), Some(0));
            assert_eq!(clist.pop_head(), None);
            assert!(clist.empty());
        }

        #[test]
        fn test_clist_peek_head() {
            let mut clist: CList<8> = CList::new();
            assert!(clist.empty());
            clist.push(0);
            clist.push(1);
            assert_eq!(clist.peek_head(), Some(0));
            assert_eq!(clist.peek_head(), Some(0));
            assert_eq!(clist.pop_head(), Some(0));
            assert_eq!(clist.peek_head(), Some(1));
            assert_eq!(clist.pop_head(), Some(1));
            assert_eq!(clist.peek_head(), None);
            assert_eq!(clist.peek_head(), None);
            assert_eq!(clist.pop_head(), None);
            assert!(clist.empty());
        }
    }
}
