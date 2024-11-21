# usb-serial

## About

This application is showing basic
[Embassy](https://github.com/embassy-rs/embassy) _USB serial_ usage with Ariel OS.

## How to run

In this folder, run

    laze build -b nrf52840dk run

With the device USB cable connected, a USB ACM serial port should show up on
your laptop / workstation.
It will simply echo every sent character.
