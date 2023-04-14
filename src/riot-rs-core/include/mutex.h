#ifndef MUTEX_H
#define MUTEX_H

#include "riot-rs-core.h"

typedef struct __attribute__((aligned(MUTEX_T_ALIGNOF))) mutex {
    char data[MUTEX_T_SIZEOF];
} mutex_t;

#define MUTEX_INIT ((mutex_t){0})
#define MUTEX_INIT_LOCKED ((mutex_t){0xff})

#endif /* MUTEX_H */
