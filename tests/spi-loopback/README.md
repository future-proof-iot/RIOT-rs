# spi-loopback

## About

This application is testing raw SPI bus loopback in RIOT-rs.

## How to run

In this folder, run

    laze build -b nrf52840dk run

This test requires MISO/MOSI directly connected for the SPI port defined in the
`pins` module.
It attempts to do a transfer and compares if what was sent has been read back.
