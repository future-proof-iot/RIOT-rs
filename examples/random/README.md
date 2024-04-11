# random

## About

This application performs a very important role in physical computing:
It produces randomness in the form of a dice roll (values 1 to 6, inclusive).
It prints out a single random value via the debug console.

## How to run

In this folder, run

    laze build -b nrf52840dk run

This will print different values most of the time on all boards it can be built
on. If there is no way to get startup entropy on a board, the example can not
be built.
