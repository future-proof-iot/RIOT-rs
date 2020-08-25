use core::mem;
use core::sync::atomic::compiler_fence;
use core::sync::atomic::Ordering;

const USIZE_BITS: usize = mem::size_of::<usize>() * 8;

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

    pub fn add(&mut self, n: usize, rq: usize) {
        assert!(n < USIZE_BITS);
        assert!(rq < N);
        compiler_fence(Ordering::AcqRel);
        self.bitcache |= 1 << rq;
        self.queues[rq] |= 1 << n;
        compiler_fence(Ordering::AcqRel);
    }

    pub fn del(&mut self, n: usize, rq: usize) {
        assert!(n < USIZE_BITS);
        assert!(rq < N);
        compiler_fence(Ordering::AcqRel);
        self.queues[rq] &= !(1 << n);
        if self.queues[rq] == 0 {
            self.bitcache &= !(1 << rq);
        }
        compiler_fence(Ordering::AcqRel);
    }

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

    pub fn advance(&mut self, pid: u8, rq: usize) {
        assert!(rq < N);
        compiler_fence(Ordering::AcqRel);
        self.queues_pos[rq] = pid;
        compiler_fence(Ordering::AcqRel);
    }
}
