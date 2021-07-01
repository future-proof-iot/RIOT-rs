#ifndef MBOX_H
#define MBOX_H

#include "msg.h"

struct __attribute__((aligned(MBOX_T_ALIGNOF))) BufferedChannel_msg_t {
    char data[MBOX_T_SIZEOF];
};

#endif /* MBOX_H */
