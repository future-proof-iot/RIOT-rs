# i2c-controller

## About

This application is testing raw I2C bus usage in RIOT-rs.

## How to run

In this folder, run

    laze build -b nrf52840dk run

This test requires an LIS3DH/LSM303AGR sensor (3-axis accelerometer) attached
to the pins configured in the `pins` module.
It attempts to read the `WHO_AM_I` register and checks the received value against the expected id.
