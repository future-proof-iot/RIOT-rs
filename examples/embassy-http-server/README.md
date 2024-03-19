# embassy-http-server

## About

This application is testing basic
[embassy](https://github.com/embassy-rs/embassy) _networking_ usage with RIOT-rs.

## How to run

In this folder, run

    laze build -b nrf52840dk run

With the device USB cable connected, a USB ethernet device should pop up.
RIOT-rs will serve an example HTML homepage at `http://10.42.0.61/` and will
expose a JSON endpoint at `http://10.42.0.61/buttons` reporting on the state of
connected push buttons if any are present, otherwise the endpoint will not be
exposed at all.

Look [here](../README.md#networking) or more information about network configuration.
