# tcp-echo

## About

This application is testing basic
[Embassy](https://github.com/embassy-rs/embassy) _networking_ usage with Ariel OS.

## How to run

In this folder, run

    laze build -b nrf52840dk run

With the device USB cable connected, a USB Ethernet device should pop up.
Ariel OS will reply to ping requests on 10.42.0.61 and host a TCP service on
port 1234 that will echo the input back to the client. It can be accessed with
e.g., `telnet`:

    telnet 10.42.0.61 1234

Look [here](../README.md#networking) or more information about network configuration.
