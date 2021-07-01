#ifndef SCHED_H
#define SCHED_H

#include "riot-rs-core.h"

#define KERNEL_PID_UNDEF THREADS_NUMOF
#define KERNEL_PID_FIRST 0
#define KERNEL_PID_LAST THREADS_NUMOF

/* TODO: ensure this is correct */
#define PRIkernel_pid "c"

#endif /* SCHED_H */
