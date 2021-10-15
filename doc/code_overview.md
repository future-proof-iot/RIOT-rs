# Introduction

This file tries to provide some links into the guts of RIOT-rs.

## src/riot-rs-core

This module contains the scheduler and IPC code, as well as RIOT C bindings /
glue code.

| file                                             | function                                              | RIOT eqivalent              |
| ------------------------------------------------ | ----------------------------------------------------- | --------------------------- |
| [thread.rs](../src/riot-rs-core/src/thread.rs)   | scheduling, task switching, thread flags, C glue code | core/sched.c, core/thread.c |
| [lock.rs](../src/riot-rs-core/src/lock.rs)       | locks                                                 | core/mutex.c                |
| [channel.rs](../src/riot-rs-core/src/channel.rs) | messaging                                             | core/msg.c, core/mbox.c     |

## src/libs

| module                                         | function                | RIOT eqivalent       |
| ---------------------------------------------- | ----------------------- | -------------------- |
| [ringbuffer](../src/lib/ringbuffer/src/lib.rs) | ringbuffer              | core/ringbuffer.c    |
| [rbi](../src/lib/rbi/src/lib.rs)               | ringbuffer index        | core/include/cib.h   |
| [clist](../src/lib/clist/src/lib.rs)           | circular intrusive list | core/include/clist.h |
