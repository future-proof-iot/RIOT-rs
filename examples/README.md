# Examples

## Overview

This directory contains example applications that showcase how to use Ariel OS.

- [benchmark/](./benchmark): how to use `benchmark()`
- [coap-server](./coap-server) and [coap-client](./coap-client): Application level networking examples
- [device-metadata/](./device-metadata): Retrieve metadata about the running device
- [hello-world/](./hello-world): a classic, async version
- [hello-world-threading/](./hello-world-threading): a classic, using a thread
- [http-server/](./http-server): HTTP server example
- [log](./log): Example demonstrating different log levels for printing feedback messages.
- [minimal/](./minimal): minimized to the max Ariel OS config
- [random/](./random): demonstrate obtaining random values
- [tcp-echo/](./tcp-echo): TCP echo example
- [thread-async-interop/](./thread-async-interop): how to make async tasks and preemptively scheduled threads interoperate
- [threading/](./threading): how to start and use preemptively scheduled threads
- [threading-event/](./threading-event): how to use `ariel_os::thread::sync::Event`
- [udp-echo/](./udp-echo): UDP echo example
- [usb-keyboard/](./usb-keyboard): USB HID example
- [usb-serial/](./usb-serial): USB serial example

## Networking

Some examples configure networking. By default, they will listen on a static
IPv4 address. Be sure to configure the host computer end of the network
accordingly.

The default IPv4 address the test examples will configure is `10.42.0.61`.
You can configure the other end like this:

    $ ip address add 10.42.0.1/24 dev <INTERFACE>
    $ ip link set up dev <INTERFACE>

To double-check that the address has indeed be added, you can use:

    $ ip address show dev <INTERFACE>

Replace `<INTERFACE>` with the name of the used network interface.
To find out the name of your interface you can use a command such as `ifconfig`.

For *USB Ethernet*, ensure that in addition to the USB cable used for flashing
and debugging, the USB device port is also connected to the host computer with
a second cable.

For *WiFi* (default on `rpi-pico-w` and the esp32 based boards), the actual WiFi
network credentials have to be supplied via environment variables:

    $ CONFIG_WIFI_NETWORK=<ssid> CONFIG_WIFI_PASSWORD=<pwd> laze build ...

In order to make the device use a DHCP client instead of the static address,
remove the `override-network-config` feature from `Cargo.toml` of the example.
