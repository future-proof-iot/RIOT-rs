# coap tests

## About

This application is a work in progress demo of running CoAP with OSCORE/EDHOC security on Ariel OS.

## Running

* Run on any board with networking, eg. `laze build -b particle-xenon run`.
* [Set up networking](../README.md).
* Run `pipx run coap-console coap://10.42.0.61 --credentials client.diag`,
  which establishes a secure CoAP connection using EDHOC and OSCORE,
  and shows the log of the device.
* Run `pipx run --spec 'aiocoap[all]' aiocoap-client coap://10.42.0.61/.well-known/core --credentials client.diag`
  to show what else the device can do.
  If you kept the log running, you will see that every new command runs through EDHOC once:
  aiocoap does not currently attempt to persist EDHOC derived OSCORE contexts across runs.
* Running multiple concurrent terminal instances is supported,
  up to the maximum number of security contexts that are stored (currently 4).
* There is also `./fauxhoc.py`, which did EDHOC manually before it was integrated in aiocoap.

## Roadmap

Eventually, all of this should be covered by 20-line examples.

Until the CoAP roadmap is complete,
this serves as a work bench, test bed, demo zone and playground at the same time.
This application will grow as parts of the roadmap are added,
and shrink as they become default or are wrapped into components of Ariel OS.
