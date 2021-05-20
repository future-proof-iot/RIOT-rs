use core::mem;
use core::sync::atomic::compiler_fence;
use core::sync::atomic::Ordering;

const USIZE_BITS: usize = mem::size_of::<usize>() * 8;

/// assumptions:
/// - runqueue numbers are 0..N (exclusive)
/// - pids are 0..USIZE_BITS (supporting max 32 threads)
/// - higher runqueue number means higher priority

// TODO: use atomics
pub struct RunQueue<const N: usize> {
    bitcache: usize,
    queues: [usize; N],
    queues_pos: [u8; N],
}

impl<const N: usize> RunQueue<{ N }> {
    pub const fn new() -> RunQueue<{ N }> {
        RunQueue {
            bitcache: 0,
            queues: [0; N],
            queues_pos: [0; N],
        }
    }

    /// add thread with pid n to runqueue number rq
    pub fn add(&mut self, n: usize, rq: usize) {
        debug_assert!(n < USIZE_BITS);
        debug_assert!(rq < N);
        compiler_fence(Ordering::AcqRel);
        self.bitcache |= 1 << rq;
        self.queues[rq] |= 1 << n;
        compiler_fence(Ordering::AcqRel);
    }

    /// remove thread with pid n from runqueue number rq
    pub fn del(&mut self, n: usize, rq: usize) {
        debug_assert!(n < USIZE_BITS);
        debug_assert!(rq < N);
        compiler_fence(Ordering::AcqRel);
        self.queues[rq] &= !(1 << n);
        if self.queues[rq] == 0 {
            self.bitcache &= !(1 << rq);
        }
        compiler_fence(Ordering::AcqRel);
    }

    /// get pid that should run next
    /// returns the next runnable thread of
    /// the runqueue with the highest index
    pub fn get_next(&self) -> u32 {
        fn ffs(val: usize) -> u32 {
            USIZE_BITS as u32 - val.leading_zeros()
        }

        compiler_fence(Ordering::AcqRel);
        let rq_ffs = ffs(self.bitcache);
        if rq_ffs > 0 {
            let rq = (rq_ffs - 1) as usize;
            let rq_pos = self.queues_pos[rq] as u32;
            let rq_unmasked = self.queues[rq];
            let rq_mask = usize::MAX.checked_shr(32 - rq_pos).unwrap_or(0);
            let rq_masked = rq_unmasked & rq_mask;
            let next = ffs(rq_masked);
            if next > 0 {
                return next - 1;
            }
            return ffs(self.queues[rq]) - 1;
        }

        return 0;
    }

    /// advance runqueue number rq
    /// (this is used to "yield" to another thread of *the same* priority)
    /// pid = current pid
    /// TODO: drop pid parameter from interface
    pub fn advance(&mut self, pid: u8, rq: usize) {
        debug_assert!(rq < N);
        compiler_fence(Ordering::AcqRel);
        self.queues_pos[rq] = pid;
        compiler_fence(Ordering::AcqRel);
    }
}

#[test_case]
fn test_rq_basic() {
    let mut runqueue: RunQueue<8> = RunQueue::new();

    runqueue.add(0, 0);
    runqueue.add(1, 0);
    runqueue.add(2, 0);

    assert!(runqueue.get_next() == 2);

    runqueue.advance(2, 0);

    assert!(runqueue.get_next() == 1);
    runqueue.advance(1, 0);

    assert!(runqueue.get_next() == 0);
    assert!(runqueue.get_next() == 0);

    runqueue.advance(1, 0);
    assert!(runqueue.get_next() == 0);

    runqueue.advance(0, 0);
    assert!(runqueue.get_next() == 2);

    runqueue.advance(2, 0);
    assert!(runqueue.get_next() == 1);
}

#[test_case]
fn test_rq_all32() {
    let mut runqueue: RunQueue<8> = RunQueue::new();

    for i in 0..=31 {
        runqueue.add(i, 0);
    }

    for i in (0..=31).rev() {
        assert!(runqueue.get_next() == i);
        runqueue.advance(i as u8, 0);
    }

    for i in (0..=31).rev() {
        assert!(runqueue.get_next() == i);
        runqueue.advance(i as u8, 0);
    }
}

#[test_case]
fn test_rq_basic_twoprio() {
    let mut runqueue: RunQueue<8> = RunQueue::new();

    runqueue.add(0, 0);
    runqueue.add(1, 0);
    runqueue.add(3, 0);

    runqueue.add(2, 1);
    runqueue.add(4, 1);

    assert!(runqueue.get_next() == 4);
    runqueue.del(4, 1);
    assert!(runqueue.get_next() == 2);
    runqueue.del(2, 1);
    assert!(runqueue.get_next() == 3);
    runqueue.del(3, 0);
    assert!(runqueue.get_next() == 1);
    runqueue.del(1, 0);
    assert!(runqueue.get_next() == 0);
    runqueue.del(0, 0);
}
