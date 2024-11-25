# random

## About

This application performs a very important role in physical computing:
It produces randomness in the form of a dice roll (values 1 to 6, inclusive).
It prints out multiple random values via the debug console.

## How to run

In this folder, run

    laze build -b nrf52840dk run

If there is no way to get startup entropy on a board, the example cannot be built.
