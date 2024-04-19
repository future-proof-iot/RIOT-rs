#![cfg_attr(not(test), no_std)]
#![feature(min_specialization)]

mod runqueue;
pub use runqueue::{CoreId, GlobalRunqueue, RunQueue, RunqueueId, ThreadId};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rq_basic() {
        let mut runqueue: RunQueue<8, 32> = RunQueue::new();

        runqueue.add(0, 0);
        runqueue.add(1, 0);
        runqueue.add(2, 0);

        assert_eq!(runqueue.get_next(0), Some(0));

        runqueue.advance(0, 0);

        assert_eq!(runqueue.get_next(0), Some(1));
        runqueue.advance(0, 0);

        assert_eq!(runqueue.get_next(0), Some(2));
        assert_eq!(runqueue.get_next(0), Some(2));

        runqueue.advance(0, 0);
        assert_eq!(runqueue.get_next(0), Some(0));

        runqueue.advance(0, 0);
        assert_eq!(runqueue.get_next(0), Some(1));

        runqueue.advance(0, 0);
        assert_eq!(runqueue.get_next(0), Some(2));
    }

    #[test]
    fn test_rq_all32() {
        let mut runqueue: RunQueue<8, 32> = RunQueue::new();

        for i in 0..=31 {
            runqueue.add(i, 0);
        }

        for i in 0..=31 {
            assert_eq!(runqueue.get_next(0), Some(i));
            runqueue.advance(0, 0);
        }

        for i in 0..=31 {
            assert_eq!(runqueue.get_next(0), Some(i));
            runqueue.advance(0, 0);
        }
    }

    #[test]
    fn test_rq_basic_twoprio() {
        let mut runqueue: RunQueue<8, 32> = RunQueue::new();

        runqueue.add(0, 0);
        runqueue.add(1, 0);
        runqueue.add(3, 0);

        runqueue.add(2, 1);
        runqueue.add(4, 1);

        assert_eq!(runqueue.get_next(0), Some(2));
        runqueue.del(2, 1);
        assert_eq!(runqueue.get_next(0), Some(4));
        runqueue.del(4, 1);
        assert_eq!(runqueue.get_next(0), Some(0));
        runqueue.del(0, 0);
        assert_eq!(runqueue.get_next(0), Some(1));
        runqueue.del(1, 0);
        assert_eq!(runqueue.get_next(0), Some(3));
        runqueue.del(3, 0);
        assert_eq!(runqueue.get_next(0), None);
    }
    #[test]
    fn test_push_twice() {
        let mut runqueue: RunQueue<8, 32> = RunQueue::new();

        runqueue.add(0, 0);
        runqueue.add(1, 0);

        assert_eq!(runqueue.get_next(0), Some(0));
        runqueue.del(0, 0);
        assert_eq!(runqueue.get_next(0), Some(1));

        runqueue.add(0, 0);

        assert_eq!(runqueue.get_next(0), Some(1));

        runqueue.advance(0, 0);
        assert_eq!(runqueue.get_next(0), Some(0));
    }

    #[test]
    fn multicore_basic() {
        let mut runqueue: RunQueue<8, 32, 4> = RunQueue::new();

        // First thread should get allocated to core 0.
        assert_eq!(runqueue.add(0, 0), Some(0));
        // Second thread should get allocated to core 1.
        assert_eq!(runqueue.add(1, 0), Some(1));

        assert_eq!(runqueue.get_next(0), Some(0));
        assert_eq!(runqueue.get_next(1), Some(1));
        assert!(runqueue.get_next(2).is_none());

        // Advancing a runqueue shouldn't change any allocations
        // if all threads in the queue are already running.
        assert_eq!(runqueue.advance(0, 0), None);
        assert_eq!(runqueue.get_next(0), Some(0));
        assert_eq!(runqueue.get_next(1), Some(1));
        assert!(runqueue.get_next(2).is_none());

        // Restores original order.
        assert_eq!(runqueue.advance(1, 0), None);

        // Add more threads, which should be allocated to free
        // cores.
        assert_eq!(runqueue.add(2, 0), Some(2));
        assert_eq!(runqueue.add(3, 0), Some(3));
        assert_eq!(runqueue.add(4, 0), None);
        assert_eq!(runqueue.get_next(0), Some(0));
        assert_eq!(runqueue.get_next(1), Some(1));
        assert_eq!(runqueue.get_next(2), Some(2));
        assert_eq!(runqueue.get_next(3), Some(3));

        // Advancing the runqueue now should change the mapping
        // on core 0, since the previous head was running there.
        assert_eq!(runqueue.advance(0, 0), Some(0));
        assert_eq!(runqueue.get_next(0), Some(4));
        // Other allocations shouldn't change.
        assert_eq!(runqueue.get_next(1), Some(1));
        assert_eq!(runqueue.get_next(2), Some(2));
        assert_eq!(runqueue.get_next(3), Some(3));

        // Adding or deleting waiting threads shouldn't change
        // any allocations.
        assert_eq!(runqueue.del(0, 0), None);
        assert_eq!(runqueue.add(5, 0), None);

        // Deleting a running thread should allocate the waiting
        // thread to the now free core.
        assert_eq!(runqueue.del(2, 0), Some(2));
        assert_eq!(runqueue.get_next(2), Some(5));
        // Other allocations shouldn't change.
        assert_eq!(runqueue.get_next(0), Some(4));
        assert_eq!(runqueue.get_next(1), Some(1));
        assert_eq!(runqueue.get_next(3), Some(3));
    }

    #[test]
    fn multicore_multiqueue() {
        let mut runqueue: RunQueue<8, 32, 4> = RunQueue::new();

        assert_eq!(runqueue.add(0, 2), Some(0));
        assert_eq!(runqueue.add(1, 2), Some(1));
        assert_eq!(runqueue.add(2, 1), Some(2));
        assert_eq!(runqueue.add(3, 0), Some(3));
        assert_eq!(runqueue.add(4, 0), None);

        assert_eq!(runqueue.get_next(0), Some(0));
        assert_eq!(runqueue.get_next(1), Some(1));
        assert_eq!(runqueue.get_next(2), Some(2));
        assert_eq!(runqueue.get_next(3), Some(3));

        // Advancing highest priority queue shouldn't change anything
        // because there are more cores than threads in this priority's queue.
        assert_eq!(runqueue.advance(0, 2), None);
        assert_eq!(runqueue.get_next(0), Some(0));
        assert_eq!(runqueue.get_next(1), Some(1));
        assert_eq!(runqueue.get_next(2), Some(2));
        assert_eq!(runqueue.get_next(3), Some(3));

        // Advancing lowest priority queue should change allocations
        // since there are two threads in this priority's queue,
        // but only one available core for them.

        // Core 3 was newly allocated.
        assert_eq!(runqueue.advance(3, 0), Some(3));
        assert_eq!(runqueue.get_next(3), Some(4));
        // Other allocations didn't change.
        assert_eq!(runqueue.get_next(0), Some(0));
        assert_eq!(runqueue.get_next(1), Some(1));
        assert_eq!(runqueue.get_next(2), Some(2));

        // Restores original order.
        runqueue.advance(4, 0);

        // Delete one high-priority thread.
        // The waiting low-priority thread should be allocated
        // to the newly freed core.

        // Core 0 was newly allocated.
        assert_eq!(runqueue.del(0, 2), Some(0));
        assert_eq!(runqueue.get_next(0), Some(4));
        // Other allocations didn't change.
        assert_eq!(runqueue.get_next(1), Some(1));
        assert_eq!(runqueue.get_next(2), Some(2));
        assert_eq!(runqueue.get_next(3), Some(3));

        // Add one medium-priority thread.
        // The low-priority thread furthest back in its priority queue
        // should be preempted.

        // Core 0 was newly allocated.
        assert_eq!(runqueue.add(5, 1), Some(0));
        assert_eq!(runqueue.get_next(0), Some(5));
        // Other allocations didn't change.
        assert_eq!(runqueue.get_next(1), Some(1));
        assert_eq!(runqueue.get_next(2), Some(2));
        assert_eq!(runqueue.get_next(3), Some(3));
    }

    #[test]
    fn multicore_invalid_core() {
        let mut runqueue: RunQueue<8, 32, 1> = RunQueue::new();
        assert_eq!(runqueue.add(0, 2), Some(0));
        assert_eq!(runqueue.add(1, 2), None);
        assert_eq!(runqueue.get_next(0), Some(0));
        assert_eq!(runqueue.get_next(0), Some(0));
        // Querying for n > `N_CORES` shouldn't cause a panic.
        assert_eq!(runqueue.get_next(1), None)
    }

    #[test]
    fn multicore_advance() {
        let mut runqueue: RunQueue<8, 32, 4> = RunQueue::new();
        assert_eq!(runqueue.add(0, 0), Some(0));
        assert_eq!(runqueue.add(1, 0), Some(1));
        assert_eq!(runqueue.add(2, 0), Some(2));
        assert_eq!(runqueue.add(3, 0), Some(3));
        assert_eq!(runqueue.add(4, 0), None);
        assert_eq!(runqueue.add(5, 0), None);

        // Advance head.
        assert_eq!(runqueue.advance(0, 0), Some(0));
        assert_eq!(runqueue.get_next(0), Some(4));
        // Other allocations didn't change.
        assert_eq!(runqueue.get_next(1), Some(1));
        assert_eq!(runqueue.get_next(2), Some(2));
        assert_eq!(runqueue.get_next(3), Some(3));

        // Advance from a thread that is not head.
        assert_eq!(runqueue.advance(2, 0), Some(2));
        assert_eq!(runqueue.get_next(2), Some(5));
        // Other allocations didn't change.
        assert_eq!(runqueue.get_next(0), Some(4));
        assert_eq!(runqueue.get_next(1), Some(1));
        assert_eq!(runqueue.get_next(3), Some(3));
    }
}
