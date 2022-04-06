#ifndef SCHED_H
#define SCHED_H

#include <inttypes.h>
#include "riot-rs-core.h"

#define KERNEL_PID_UNDEF THREADS_NUMOF
#define KERNEL_PID_FIRST 0
#define KERNEL_PID_LAST THREADS_NUMOF
#define KERNEL_PID_ISR KERNEL_PID_UNDEF

/* TODO: ensure this is correct */
#define PRIkernel_pid "d"

#endif /* SCHED_H */
