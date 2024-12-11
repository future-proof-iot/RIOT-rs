# device-metadata

## About

This application prints all device information that the application knows
about the device and the running program
at startup to the debug console.

## How to run

In this folder, run

    laze build -b nrf52840dk run

## Example output

When run, this example prints the board name and the device id, if available.
For example:

    INFO  Available information:
    INFO  Board type: nrf52840dk
    INFO  Device ID: [80, 6a, ec, 55, 8c, c2, 43, 8e]
    INFO  Device's first EUI-48 address: 02:9a:05:d7:38:e9
