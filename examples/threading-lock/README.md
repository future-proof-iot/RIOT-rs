# Threading Lock

## About

This application demonstrates how multiple threads can wait for the same lock and get unblocked in FIFO order.

## How to run

In this folder, run

    laze build -b nrf52840dk run

The application will start an async task that acquires a lock and holds it for a couple of seconds, and two threads that wait for the same lock.
