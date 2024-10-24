# threading multicore

## About

This application demonstrates basic threading on multicore.

## How to run

In this folder, run

    laze build -b rpi-pico run

The application will start two threads that each print their ID and the core that they are running on, before
entering a busy loop.
The thread that is started first in pinned to Core 1 using core affinities.
