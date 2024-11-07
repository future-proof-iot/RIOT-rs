# Threading Events

## About

This application demonstrates the usage of an `Event` as synchronization method for
threads.

## How to run

In this folder, run

    laze build -b nrf52840dk run

The application will start five threads with different priorities. All but one
threads block on a global `Event` until the one thread sets it.
