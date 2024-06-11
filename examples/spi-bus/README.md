# spi-bus

## About

This application is demonstrating raw SPI bus usage in RIOT-rs.
Please use `riot_rs::sensors` instead for a high-level sensor abstraction that is architecture-agnostic.

## How to run

In this folder, run

    laze build -b nrf52840dk run

This example requires a LIS3DH sensor (3-axis accelerometer) attached to the pins configured in the example.
