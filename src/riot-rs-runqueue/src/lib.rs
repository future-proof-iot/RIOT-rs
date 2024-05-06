#![cfg_attr(not(test), no_std)]

mod runqueue;
pub use runqueue::{RunQueue, RunqueueId, ThreadId};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rq_basic() {
        let mut runqueue: RunQueue<8, 32> = RunQueue::new();

        runqueue.add(ThreadId(0), RunqueueId(0));
        runqueue.add(ThreadId(1), RunqueueId(0));
        runqueue.add(ThreadId(2), RunqueueId(0));

        assert_eq!(runqueue.get_next(), Some(ThreadId(0)));

        runqueue.advance(RunqueueId(0));

        assert_eq!(runqueue.get_next(), Some(ThreadId(1)));
        runqueue.advance(RunqueueId(0));

        assert_eq!(runqueue.get_next(), Some(ThreadId(2)));
        assert_eq!(runqueue.get_next(), Some(ThreadId(2)));

        runqueue.advance(RunqueueId(0));
        assert_eq!(runqueue.get_next(), Some(ThreadId(0)));

        runqueue.advance(RunqueueId(0));
        assert_eq!(runqueue.get_next(), Some(ThreadId(1)));

        runqueue.advance(RunqueueId(0));
        assert_eq!(runqueue.get_next(), Some(ThreadId(2)));
    }

    #[test]
    fn test_rq_all32() {
        let mut runqueue: RunQueue<8, 32> = RunQueue::new();

        for i in 0..=31 {
            runqueue.add(ThreadId(i), RunqueueId(0));
        }

        for i in 0..=31 {
            assert_eq!(runqueue.get_next(), Some(ThreadId(i)));
            runqueue.advance(RunqueueId(0));
        }

        for i in 0..=31 {
            assert_eq!(runqueue.get_next(), Some(ThreadId(i)));
            runqueue.advance(RunqueueId(0));
        }
    }

    #[test]
    fn test_rq_basic_twoprio() {
        let mut runqueue: RunQueue<8, 32> = RunQueue::new();

        runqueue.add(ThreadId(0), RunqueueId(0));
        runqueue.add(ThreadId(1), RunqueueId(0));
        runqueue.add(ThreadId(3), RunqueueId(0));

        runqueue.add(ThreadId(2), RunqueueId(1));
        runqueue.add(ThreadId(4), RunqueueId(1));

        assert_eq!(runqueue.get_next(), Some(ThreadId(2)));
        runqueue.del(ThreadId(2), RunqueueId(1));
        assert_eq!(runqueue.get_next(), Some(ThreadId(4)));
        runqueue.del(ThreadId(4), RunqueueId(1));
        assert_eq!(runqueue.get_next(), Some(ThreadId(0)));
        runqueue.del(ThreadId(0), RunqueueId(0));
        assert_eq!(runqueue.get_next(), Some(ThreadId(1)));
        runqueue.del(ThreadId(1), RunqueueId(0));
        assert_eq!(runqueue.get_next(), Some(ThreadId(3)));
        runqueue.del(ThreadId(3), RunqueueId(0));
        assert_eq!(runqueue.get_next(), None);
    }
    #[test]
    fn test_push_twice() {
        let mut runqueue: RunQueue<8, 32> = RunQueue::new();

        runqueue.add(ThreadId(0), RunqueueId(0));
        runqueue.add(ThreadId(1), RunqueueId(0));

        assert_eq!(runqueue.get_next(), Some(ThreadId(0)));
        runqueue.del(ThreadId(0), RunqueueId(0));
        assert_eq!(runqueue.get_next(), Some(ThreadId(1)));

        runqueue.add(ThreadId(0), RunqueueId(0));

        assert_eq!(runqueue.get_next(), Some(ThreadId(1)));

        runqueue.advance(RunqueueId(0));
        assert_eq!(runqueue.get_next(), Some(ThreadId(0)));
    }
}
