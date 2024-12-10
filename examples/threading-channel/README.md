# Threading Channel

## About

This application demonstrates the usage of a
[`Channel`](https://ariel-os.github.io/ariel-os/dev/docs/api/ariel_os/thread/sync/struct.Channel.html)
for message passing between threads.

The application will start two threads: thread 0 and thread 1.
On start, thread 0 sends a message through the channel.
Thread 1 will wait for a message on the channel and print the received value.

## How to run

In this folder, run

    laze build -b nrf52840dk run

## Example output

When run on a board, this example shows the following output:

    INFO  [Thread ThreadId(1)] Waiting to receive a message...
    INFO  [Thread ThreadId(0)] Sending a message...
    INFO  [Thread ThreadId(1)] The answer to the Ultimate Question of Life, the Universe, and Everything is: 42.
