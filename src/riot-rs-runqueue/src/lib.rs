#![cfg_attr(not(test), no_std)]

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
        runqueue.pop_head(ThreadId::new(2), RunqueueId::new(1));
        assert_eq!(runqueue.get_next(), Some(ThreadId::new(4)));
        runqueue.pop_head(ThreadId::new(4), RunqueueId::new(1));
        assert_eq!(runqueue.get_next(), Some(ThreadId::new(0)));
        runqueue.pop_head(ThreadId::new(0), RunqueueId::new(0));
        assert_eq!(runqueue.get_next(), Some(ThreadId::new(1)));
        runqueue.pop_head(ThreadId::new(1), RunqueueId::new(0));
        assert_eq!(runqueue.get_next(), Some(ThreadId::new(3)));
        runqueue.pop_head(ThreadId::new(3), RunqueueId::new(0));
        assert_eq!(runqueue.get_next(), None);
    }
    #[test]
    fn test_push_twice() {
        let mut runqueue: RunQueue<8, 32> = RunQueue::new();

        runqueue.add(ThreadId::new(0), RunqueueId::new(0));
        runqueue.add(ThreadId::new(1), RunqueueId::new(0));

        assert_eq!(runqueue.get_next(), Some(ThreadId::new(0)));
        runqueue.pop_head(ThreadId::new(0), RunqueueId::new(0));
        assert_eq!(runqueue.get_next(), Some(ThreadId::new(1)));

        runqueue.add(ThreadId::new(0), RunqueueId::new(0));

        assert_eq!(runqueue.get_next(), Some(ThreadId::new(1)));

        runqueue.advance(RunqueueId::new(0));
        assert_eq!(runqueue.get_next(), Some(ThreadId::new(0)));
    }

    #[test]
    fn test_rq_del() {
        let mut runqueue: RunQueue<8, 32> = RunQueue::new();

        runqueue.add(ThreadId::new(0), RunqueueId::new(1));
        runqueue.add(ThreadId::new(1), RunqueueId::new(1));
        runqueue.add(ThreadId::new(2), RunqueueId::new(0));

        assert_eq!(runqueue.get_next(), Some(ThreadId::new(0)));

        // Delete thread that isn't head.
        runqueue.del(ThreadId::new(1));
        assert_eq!(runqueue.get_next(), Some(ThreadId::new(0)));

        // Delete head.
        runqueue.del(ThreadId::new(0));
        assert_eq!(runqueue.get_next(), Some(ThreadId::new(2)));

        // Delete invalid thread.
        runqueue.del(ThreadId::new(3));
        assert_eq!(runqueue.get_next(), Some(ThreadId::new(2)));

        // Delete last thread in runqueue.
        runqueue.del(ThreadId::new(2));
        assert_eq!(runqueue.get_next(), None);
    }

    #[test]
    fn iter() {
        let mut runqueue: RunQueue<8, 32> = RunQueue::new();

        runqueue.add(ThreadId::new(0), RunqueueId::new(0));
        runqueue.add(ThreadId::new(1), RunqueueId::new(2));
        runqueue.add(ThreadId::new(2), RunqueueId::new(2));
        runqueue.add(ThreadId::new(3), RunqueueId::new(3));

        let (head, rq) = runqueue.get_next_with_rq().unwrap();
        assert_eq!(head, ThreadId::new(3));
        assert_eq!(rq, RunqueueId::new(3));
        let mut iter = runqueue.iter_from(head, rq);

        assert_eq!(iter.next(), Some(ThreadId::new(1)));
        assert_eq!(iter.next(), Some(ThreadId::new(2)));
        assert_eq!(iter.next(), Some(ThreadId::new(0)));

        assert!(iter.next().is_none());
        assert!(iter.next().is_none());

        let mut iter2 = runqueue.iter_from(ThreadId::new(1), RunqueueId::new(2));
        assert_eq!(iter2.next(), Some(ThreadId::new(2)));
        assert_eq!(iter2.next(), Some(ThreadId::new(0)));
        assert!(iter2.next().is_none());
    }

    #[test]
    fn filter() {
        let mut runqueue: RunQueue<8, 32> = RunQueue::new();

        runqueue.add(ThreadId::new(0), RunqueueId::new(0));
        runqueue.add(ThreadId::new(1), RunqueueId::new(2));
        runqueue.add(ThreadId::new(2), RunqueueId::new(2));
        runqueue.add(ThreadId::new(3), RunqueueId::new(3));

        assert_eq!(runqueue.get_next_filter(|_| true), Some(ThreadId::new(3)));
        assert_eq!(runqueue.get_next_filter(|_| false), None);
        assert_eq!(
            runqueue.get_next_filter(|t| *t == ThreadId::new(0)),
            Some(ThreadId::new(0))
        );
        assert_eq!(
            runqueue.get_next_filter(|t| *t != ThreadId::new(0)),
            Some(ThreadId::new(3))
        );
        assert_eq!(
            runqueue.get_next_filter(|t| usize::from(*t) % 2 == 0),
            Some(ThreadId::new(2))
        );
        assert_eq!(
            runqueue.get_next_filter(|t| usize::from(*t) < 2),
            Some(ThreadId::new(1))
        );
    }
}
