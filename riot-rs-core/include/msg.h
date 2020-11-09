#ifndef MSG_H
#define MSG_H

#include "riot-rs-core.h"

#define KERNEL_PID_ISR (kernel_pid_t)(-1)
#define msg_send_int msg_send
#define msg_try_send msg_send

#endif /* MSG_H */
