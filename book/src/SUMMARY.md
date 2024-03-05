# Summary

- [Introduction](./introduction.md)

# User Guide

## Multi-threading

Riot-rs implements a scheduler based on fixed priorities and preemption.
Within the same priority level, threads are scheduled cooperatively.

Threading can be enabled with the `threading` feature.
If the feature is enabled, at least one thread **must** be defined.  
Threads should be implemented using the `riot_rs_macros::thread` proc macro, which takes care of calling the necessary initialization methods and linking the thread function element it into the binary.
A `ThreadId` between 0 and `riot_rs_threads::THREADS_NUMOF` is assigned to each thread in the order in which the threads are declared (*TODO: is this a defined behavior? Would ease e.g. setting thread\_flags for other threads because the ids are known at compile time, but is it really always ensured?*).
Optionally, the stacksize and a priority between 1 and `riot_rs_threads::SCHED_PRIO_LEVELS` can be configured. Per default, the stack size is 2048 bytes and priority is 1.

The `riot_rs_threading` crate supports basic synchronization primitives:
- a synchronous (blocking) channel for sending data between threads (the data must implement `Copy`)
- a lock that can be used to implement mutually exclusive access to a resource
- a thread-flags implementation that enables threads to wait for specific flags
  (*TODO: should the thread\_flags even be mentioned here/ are they meant to be used by users?*)

### Example

```rs
static CHANNEL: Channel<u8> = riot_rs::thread::channel::Channel::new();

#[riot_rs::thread]
fn thread0() {
    println!("Hello from thread 0.");
    CHANNEL.send(&42);
}

#[riot_rs::thread(stacksize = 4096, priority = 2)]
fn thread1() {
    println!("Hello from thread 1.");
    let recv = CHANNEL.recv();
    println!("The answer to the Ultimate Question of Life, the Universe, and Everything is {}.", recv);
}
```

(*TODO*)

# Developer Guide

## riot-rs-threads

The `riot-rs-threads` crate implements multi-threading and the scheduler.

It makes heavy use of the `critical_section` crate to ensure that there are no conflicting accesses to the shared thread state.

Threads are managed through the static `Threads` structure, that contains all thread data, runqueues, and state information.
The `EnsureOnce` wrapper around it uses a [`critical_section::Mutex`](https://docs.rs/critical-section/latest/critical_section/struct.Mutex.html) to ensures that each access is marked as a [`critical_section`](https://doc.rust-lang.org/std/cell/struct.RefCell.html), and that a reference is dropped directly after the access.

Thread data is stored in the `Thread` structure.
Apart from general management info like the `ThreadId` and runqueue number (`RunqueueId`), it stores the thread's execution state after a context switch.
A context switch may happen after the scheduler was triggered.
On ARM Cortex-M, it is initiated through a PendSV exception, by calling the public function `riot_rs_threads::arch::schedule()`.

### Scheduling

The scheduler is triggered in the following cases:
- The current thread is blocked on a resource.
- The current thread cooperatively yields to another thread with the same priority.
- The current thread sleeps.
- The current thread has run to completion.
- Another thread was unblocked on a resource or woken up from sleep.

Triggering the scheduler doesn't necessarily imply a thread switch.
In particular in the last case, the switch only occurs if the now unblocked thread has a higher priority than the current one.

The runqueue lives in a separate crate `riot_rs_runqueue`.
It is implemented as circular linked lists for each priority level, to which `ThreadId`s can be added and removed.
The runqueue always returns the head from the highest-priority list.
Within a priority list, the head can be advanced if a thread cooperatively yields.

If the scheduler is triggered and all runqueues are empty, sleep mode is entered until an interrupt occurs.

### Context Switching

(*TODO: is this arch-specific or always the case?*)

Context switching is implemented in the `riot_rs-threads::arch` module.

Following the initial PendSV exception (for Cortex-M), the arch-specific exception handler calls the scheduler to prompt for a context switch.  
If a context switch is required, the following steps occur:
1. the scheduler:
   - updates the state in `Threads`
   - stores the stack pointer of the old thread
   - returns:
      - pointer to memory for storing the register state of the old thread
      - pointer to memory location for loading the register state of the new thread
      - the new stack pointer
2. the arch-specific handler:
   - stores the current register state
   - loads the new register state
   - updates the stack pointer
3. return from exception

### Thread Creation

(*TODO*)

- [Appendices](./appendices.md)
  - [Coding Conventions](./coding-conventions.md)
