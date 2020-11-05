#ifndef MUTEX_H
#define MUTEX_H

#include "riot-rs-core.h"

struct __attribute__((aligned(MUTEX_T_ALIGNOF))) Lock {
    char data[MUTEX_T_SIZEOF];
};

typedef struct Lock mutex_t;
#define MUTEX_INIT ((mutex_t){0})

#endif /* MUTEX_H */
