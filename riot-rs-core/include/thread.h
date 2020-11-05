#ifndef THREAD_H
#define THREAD_H

#include "riotcore-rs.h"
#include "cpu_conf.h"
#include "thread_config.h"

static inline uint8_t thread_create(char *stack_ptr,
                                    uintptr_t stack_size,
                                    uint8_t priority,
                                    uint32_t flags,
                                    void *(*thread_func)(void *),
                                    void *arg,
                                    const char *_name)
{
    return _thread_create(stack_ptr, stack_size, SCHED_PRIO_LEVELS - priority, flags,
                          (uintptr_t)thread_func, (uintptr_t)arg, _name);
}

#endif /* THREAD_H */
