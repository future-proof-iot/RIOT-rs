# gpio

## About

This application demonstrates GPIO initialization and usage.

Two GPIOs are initialized at the start of the example.
One GPIO is configured as output and used to drive an LED on the board.
The second GPIO is configured to read a button on the board.

Depending on the board design,
pressing the button makes the LED blink faster or slower.

The configuration of which GPIO is used for the LED
and of which GPIO is used for reading the button state
is in the [`pins`](./src/pins.rs) module.

## How to run

In this folder, run

    laze build -b nrf52840dk run

## Example output

This example does not output anything on the console.
The only output from this example is via the LED on the board.
