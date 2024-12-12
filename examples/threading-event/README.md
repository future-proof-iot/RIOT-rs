# Threading Events

## About

This application demonstrates the usage of an
[`Event`](https://ariel-os.github.io/ariel-os/dev/docs/api/ariel_os/thread/sync/struct.Event.html)
as synchronization method for threads.

The example starts five threads (0 through 4).
A static `Event` is created as synchronization mechanism for the threads.
All threads wait for the event to get set,
printing output before and after the event is set.
Only thread 4 is different in that
it sets the event before waiting on that same event.
All threads must thus wait for thread 4 to set the event
before they can continue.

## How to run

In this folder, run

    laze build -b nrf52840dk run

The application will start five threads with different priorities.
All but one threads block on a global `Event` until the one thread sets it.

## Example output

When run, this example shows the following output:

    INFO  [ThreadId(3)@RunqueueId(3)] Waiting for event...
    INFO  [ThreadId(2)@RunqueueId(2)] Waiting for event...
    INFO  [ThreadId(1)@RunqueueId(1)] Waiting for event...
    INFO  [ThreadId(4)@RunqueueId(1)] Setting event...
    INFO  [ThreadId(3) Done.
    INFO  [ThreadId(2) Done.
    INFO  [ThreadId(4)@RunqueueId(1)] Event set.
    INFO  [ThreadId(4)@RunqueueId(1)] Waiting for event...
    INFO  [ThreadId(4) Done.
    INFO  [ThreadId(1) Done.
    INFO  [ThreadId(0)@RunqueueId(0)] Waiting for event...
    INFO  [ThreadId(0) Done.
    INFO  All five threads should have reported "Done.". exiting.
