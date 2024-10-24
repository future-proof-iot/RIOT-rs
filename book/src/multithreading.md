# Multithreading

RIOT-rs supports multithreading on the Cortex-M, RISC-V, and Xtensa architectures.

## Scheduling

- **Fixed-priority, preemptive scheduling** policy with up to 32 supported priority levels.
- **Thread priorities are dynamic** and can be changed at runtime.
- The Runqueue is implemented as a circular linked list for each priority, requiring n × m × 8 bits for _n_ maximum threads and _m_ supported priorities. All operations on the runqueue are performed in constant time, except for the deletion of a thread that is not the head of the runqueue.
- **Deep sleep when idle**: On single core, no idle threads are created. Instead, if the runqueue is empty, the processor enters deep sleep until a next thread is ready. The context of the previously running thread is only saved once the next thread is ready and the context switch occurs.
- **Same priority threads are scheduled cooperatively.** The scheduler itself is tickless, therefore time-slicing isn't supported. [Timers](https://docs.rs/embassy-time/latest/embassy_time/struct.Timer.html) are still supported through the timer API from the integrated [Embassy] crate.

### Thread Creation

- Threads can either be declared using a macro, which creates and starts the thread during startup, or spawned dynamically at runtime. In the latter case, the stack memory must still be statically allocated at compile time.
- The **maximum number of threads** is defined with a constant value at compile time. This maximum limits the number of concurrently running threads, but it is still possible to create more threads if earlier ones have finished their execution.
- Multiple **asynchronous Tasks** can be spawned within each thread with an executor from the integrated [Embassy] crate. This bridges the gap with async Rust, future-based concurrency, and asynchronous I/O. The executor executes all its tasks inside the thread context. When all tasks on the executor are pending, the owning thread is suspended.

## Synchronization Primitives

- **Locks** in RIOT-rs are basic non-reentrant locking objects that carry no data and serve as a building block for other synchronization primitives.
- **Mutexes** are the user-facing variant of locks that wrap shared objects and provide mutual exclusion when accessing the inner data.  The mutexes implement the priority inheritance protocol to prevent priority inversion if a higher priority thread is blocked on a mutex that is locked by a lower priority thread. The access itself is realized through a `MutexGuard`. If the _Guard_ goes out of scope, the mutex is automatically released.
- **Channels** facilitate the synchronous transfer of data between threads.  They are not bound to one specific sender or receiver, but only one sender and one receiver are possible at a time.
- **Thread flags** can be set per thread at any time. A thread is blocked until the flags it is waiting for have been set, which also includes flags that have been set prior to the _wait_ call.

**All of the above synchronization primitives are blocking**. When a thread is blocked, its execution is paused, allowing the CPU to be freed for other tasks. If multiple threads are blocked on the same object, they are entered into a waitlist that is sorted by priority and FIFO order.

## Multicore Support

RIOT-rs optionally supports symmetric multiprocessing (SMP).
- **Supported dual-core Chips** where SMP is enabled by default:
    - ESP32-S3
    - RP2040
- **Porting from single-core to multicore** requires no changes in the user-application.
- **One global runqueue** is shared among all cores. The scheduler assigns the _n_ highest-priority, ready, and non-conflicting threads to the _n_ available cores. The scheduler can be invoked individually on each core. Whenever a higher priority thread becomes ready, the scheduler is triggered on the core with the lowest-priority running thread to perform a context switch.
- **Core affinities** are optionally supported. They can be set per thread to restrict the thread's execution to one or multiple specific processors.
- **One Idle Thread per core** is created on multicore system. This helps avoid conflicts that could occur on multicore if deep sleep is entered directly from within the scheduler. When the idle thread is running, it then prompts the current core to enter deep sleep.

## Mutual Exclusion in the Kernel

RIOT-rs uses a single global critical section for all kernel operations.
- **On single-core** this critical section is implemented by masking all interrupts.
- **On multicore** an additional hardware spinlock is used in the case of the RP2040, and atomics are used on the ESP32-S3 to ensure that the critical section is enforced on all cores.

[Embassy]: https://embassy.dev/
