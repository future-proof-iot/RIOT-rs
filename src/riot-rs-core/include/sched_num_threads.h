#ifndef SCHED_NUM_THREADS_H
#define SCHED_NUM_THREADS_H

/* TODO: horrible hack. APIfy sched_num_threads */
#define AtomicUsize size_t
#define sched_num_threads THREADS_IN_USE

#endif /* SCHED_NUM_THREADS_H */
