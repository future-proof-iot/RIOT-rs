# threading multicore

## About

This application demonstrates basic threading on multicore.

## How to run

In this folder, run

    laze build -b rpi-pico-w run

The application will start multiple threads with different priorities and core affinities.
The threads print their ID and the core they are running on.
