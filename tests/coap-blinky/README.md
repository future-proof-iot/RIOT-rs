# coap blinky

## About

This application makes the GPIO pin from the blinky example accessible over the network.

## Running

* Run on any board with networking, eg. `laze build -b particle-xenon run`.
* [Set up networking](../README.md).
* Run `pipx run --spec 'aiocoap[all]' aiocoap-client coap://10.42.0.61/led -m PUT --content-format application/cbor --payload true`
  or `false.

## Roadmap

Right now, this demonstrates how easily code written for RIOT OS can be shared with Ariel OS.

On the long run, exposed LEDs should be distinguished from GPIO pins,
exposed GPIO pins should be configurable in their direction,
and a good default policy for this application needs to be found.
