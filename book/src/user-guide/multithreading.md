# Multithreading

Riot-rs implements a scheduler based on fixed priorities and preemption.
This means that the highest priority thread is always running.
Within one priority level, threads are scheduled cooperatively.

Threading can be enabled with the `threading` feature.
If the feature is enabled, at least one thread **must** be defined.  
Threads should be implemented using the `riot_rs_macros::thread` proc macro, which takes care of calling the necessary initialization methods and linking the thread function element it into the binary.
A `ThreadId` between 0 and `riot_rs_threads::THREADS_NUMOF` is assigned to each thread in the order in which the threads are declared (*TODO: is this a defined behavior? Would ease e.g. setting thread\_flags for other threads because the ids are known at compile time, but is it really always ensured?*).
Optionally, the stacksize and a priority between 1 and `riot_rs_threads::SCHED_PRIO_LEVELS` can be configured. Per default, the stack size is 2048 bytes and priority is 1.

The `riot_rs_threading` crate supports basic synchronization primitives:
- a synchronous (blocking) channel for sending data between threads (the data must implement `Copy`)
- a lock that can be used to implement mutually exclusive access to a resource
- a thread-flags implementation that enables threads to wait for specific flags

## Example

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
