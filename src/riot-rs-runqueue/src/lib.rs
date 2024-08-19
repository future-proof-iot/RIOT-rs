#![cfg_attr(not(test), no_std)]
#![feature(lint_reasons)]

mod runqueue;
pub use runqueue::{RunQueue, RunqueueId, ThreadId};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rq_basic() {
        let mut runqueue: RunQueue<8, 32> = RunQueue::new();

        runqueue.add(ThreadId::new(0), RunqueueId::new(0));
        runqueue.add(ThreadId::new(1), RunqueueId::new(0));
        runqueue.add(ThreadId::new(2), RunqueueId::new(0));

        assert_eq!(runqueue.get_next(), Some(ThreadId::new(0)));

        runqueue.advance(RunqueueId::new(0));

        assert_eq!(runqueue.get_next(), Some(ThreadId::new(1)));
        runqueue.advance(RunqueueId::new(0));

        assert_eq!(runqueue.get_next(), Some(ThreadId::new(2)));
        assert_eq!(runqueue.get_next(), Some(ThreadId::new(2)));

        runqueue.advance(RunqueueId::new(0));
        assert_eq!(runqueue.get_next(), Some(ThreadId::new(0)));

        runqueue.advance(RunqueueId::new(0));
        assert_eq!(runqueue.get_next(), Some(ThreadId::new(1)));

        runqueue.advance(RunqueueId::new(0));
        assert_eq!(runqueue.get_next(), Some(ThreadId::new(2)));
    }

    #[test]
    fn test_rq_all32() {
        let mut runqueue: RunQueue<8, 32> = RunQueue::new();

        for i in 0..=31 {
            runqueue.add(ThreadId::new(i), RunqueueId::new(0));
        }

        for i in 0..=31 {
            assert_eq!(runqueue.get_next(), Some(ThreadId::new(i)));
            runqueue.advance(RunqueueId::new(0));
        }

        for i in 0..=31 {
            assert_eq!(runqueue.get_next(), Some(ThreadId::new(i)));
            runqueue.advance(RunqueueId::new(0));
        }
    }

    #[test]
    fn test_rq_basic_twoprio() {
        let mut runqueue: RunQueue<8, 32> = RunQueue::new();

        runqueue.add(ThreadId::new(0), RunqueueId::new(0));
        runqueue.add(ThreadId::new(1), RunqueueId::new(0));
        runqueue.add(ThreadId::new(3), RunqueueId::new(0));

        runqueue.add(ThreadId::new(2), RunqueueId::new(1));
        runqueue.add(ThreadId::new(4), RunqueueId::new(1));

        assert_eq!(runqueue.get_next(), Some(ThreadId::new(2)));
        runqueue.del(ThreadId::new(2), RunqueueId::new(1));
        assert_eq!(runqueue.get_next(), Some(ThreadId::new(4)));
        runqueue.del(ThreadId::new(4), RunqueueId::new(1));
        assert_eq!(runqueue.get_next(), Some(ThreadId::new(0)));
        runqueue.del(ThreadId::new(0), RunqueueId::new(0));
        assert_eq!(runqueue.get_next(), Some(ThreadId::new(1)));
        runqueue.del(ThreadId::new(1), RunqueueId::new(0));
        assert_eq!(runqueue.get_next(), Some(ThreadId::new(3)));
        runqueue.del(ThreadId::new(3), RunqueueId::new(0));
        assert_eq!(runqueue.get_next(), None);
    }
    #[test]
    fn test_push_twice() {
        let mut runqueue: RunQueue<8, 32> = RunQueue::new();

        runqueue.add(ThreadId::new(0), RunqueueId::new(0));
        runqueue.add(ThreadId::new(1), RunqueueId::new(0));

        assert_eq!(runqueue.get_next(), Some(ThreadId::new(0)));
        runqueue.del(ThreadId::new(0), RunqueueId::new(0));
        assert_eq!(runqueue.get_next(), Some(ThreadId::new(1)));

        runqueue.add(ThreadId::new(0), RunqueueId::new(0));

        assert_eq!(runqueue.get_next(), Some(ThreadId::new(1)));

        runqueue.advance(RunqueueId::new(0));
        assert_eq!(runqueue.get_next(), Some(ThreadId::new(0)));
    }
}
