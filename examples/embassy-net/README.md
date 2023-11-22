# embassy-net

## About

This application is testing basic
[embassy](https://github.com/embassy-rs/embassy) _networking_ usage with RIOT-rs.

## How to run

In this folder, run

    laze build -b nrf52840dk run

With the device USB cable connected, a USB ethernet device should pop up.
RIOT-rs will reply to ping requests on 10.0.42.61.
