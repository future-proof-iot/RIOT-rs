# threading

## About

This application demonstrates starting threads.
It starts two threads, each of them prints a message.
The two threads are started via the
[`#[thread]` macro](https://ariel-os.github.io/ariel-os/dev/docs/api/ariel_os/attr.thread.html).

## How to run

In this folder, run

    laze build -b nrf52840dk run

## Expected output

The output of this example is:

    INFO  Hello from thread 1
    INFO  Hello from thread 0
