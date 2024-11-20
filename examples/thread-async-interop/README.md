# thread-async-interop

## About

This application is showing how to make async tasks and preemptively scheduled
threads interoperate using synchronization primitives that work for both.

This application manually starts an async **task** from a **thread**. This task
signals a new value on a [`Signal`] every 100 milliseconds. The thread blocks
on this signal via [`block_on`] causing it to sleep until a new value is
available. The resulting value and the time is printed for every value
returned. After 10 values received, the main thread exits.

## How to run

In this folder, run

    laze build -b nrf52840dk run

## Expected output

    INFO  main(): starting
    INFO  async_task(): starting
    INFO  async_task(): signalling, counter=0
    INFO  main(): now=0ms threadtest() counter=0
    INFO  async_task(): signalling, counter=1
    INFO  main(): now=100ms threadtest() counter=1
    INFO  async_task(): signalling, counter=2
    INFO  main(): now=200ms threadtest() counter=2
    INFO  async_task(): signalling, counter=3
    INFO  main(): now=300ms threadtest() counter=3
    INFO  async_task(): signalling, counter=4
    INFO  main(): now=400ms threadtest() counter=4
    INFO  async_task(): signalling, counter=5
    INFO  main(): now=500ms threadtest() counter=5
    INFO  async_task(): signalling, counter=6
    INFO  main(): now=600ms threadtest() counter=6
    INFO  async_task(): signalling, counter=7
    INFO  main(): now=700ms threadtest() counter=7
    INFO  async_task(): signalling, counter=8
    INFO  main(): now=800ms threadtest() counter=8
    INFO  async_task(): signalling, counter=9
    INFO  main(): now=900ms threadtest() counter=9
    INFO  main(): all good, exiting.

[`signal`]: https://ariel-os.github.io/RIOT-rs/dev/docs/api/embassy_sync/signal/struct.Signal.html
[`block_on`]: https://ariel-os.github.io/RIOT-rs/dev/docs/api/riot_rs/blocker/fn.block_on.html
