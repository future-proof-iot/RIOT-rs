# riot-rs-threads

The `riot-rs-threads` crate implements multi-threading and the scheduler.

It makes heavy use of the `critical_section` crate to ensure that there are no conflicting accesses to the shared thread state.

Threads are managed through the static `Threads` structure, that contains all thread data, runqueues, and state information.
The `EnsureOnce` wrapper around it uses a [`critical_section::Mutex`](https://docs.rs/critical-section/latest/critical_section/struct.Mutex.html) to ensures that each access is marked as a [`critical_section`](https://doc.rust-lang.org/std/cell/struct.RefCell.html), and that a reference is dropped directly after the access.

Thread data is stored in the `Thread` structure.
Apart from general management info like the `ThreadId` and runqueue number (`RunqueueId`), it stores the thread's execution state after a context switch.
A context switch may happen after the scheduler was triggered.
On ARM Cortex-M, it is initiated through a PendSV exception, by calling the public function `riot_rs_threads::arch::schedule()`.

## Scheduling

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

## Context Switching

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

## Creating Threads

Threads are created using `riot_rs_thread::thread_create`.
Apart from a pointer to the thread function, the first argument and the priority, it requires a pointer to the thread's stack as input.
The stack can be allocated using the [`static_cell`](https://docs.rs/static_cell/latest/static_cell/) crate, which allows to reserve memory at compile time that can then be initialized at runtime.

`thread_create` sets up the thread's stack and adds it to the runqueue.
Setting up the stack is arch specific and realized in the exact configuration as the CPU's interrupt-service routine does it when a running thread is interrupted by a context switch, so they can be restored after the ISR returns.
Apart from setting up the first argument and PC for the thread function, it also sets up the link-register with a cleanup function that will run once the user's function returned.

The user-side logic has to be implemented in a separate function and added to the `riot_rs_thread::THREAD_FNS` distributed slice.
A [`linkme::distributed_slice`](https://docs.rs/linkme/latest/linkme/struct.DistributedSlice.html) allows to declare a static slice of elements that is then linked into a contiguous section of the binary.
In `riot_rs`, distributed slices are used to inject initialization functions from the outside into the start-up code.
For convenience, the `riot_rs_macros::thread` macro is implements the above boilerplate code and can be used directly on the function that should run inside the thread.
