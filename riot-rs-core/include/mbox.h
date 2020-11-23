#ifndef MBOX_H
#define MBOX_H

#include "msg.h"

struct __attribute__((aligned(MBOX_T_ALIGNOF))) Channel_msg_t {
    char data[MBOX_T_SIZEOF];
};

//#define MUTEX_INIT ((mutex_t){0})
#endif /* MBOX_H */
