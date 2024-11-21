# spi-main

## About

This application is testing raw SPI bus usage in Ariel OS.

## How to run

In this folder, run

    laze build -b nrf52840dk run

This test requires an LIS3DH sensor (3-axis accelerometer) attached to the pins configured in the
`pins` module.
It attempts to read the `WHO_AM_I` register and checks the received value against the expected id.
