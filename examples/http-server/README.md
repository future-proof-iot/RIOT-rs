# http-server

## About

This application demonstrates running an HTTP server with Ariel OS.

## How to run

In this folder, run

    laze build -b nrf52840dk run

Ariel OS will serve an example HTML homepage at <http://10.42.0.61/> and will
expose a JSON endpoint at <http://10.42.0.61/button> reporting on the state of
a connected push button if present, otherwise the endpoint will not be exposed
at all.

Look [here](../README.md#networking) or more information about network configuration.
