# coap demo

## About

This application is a work in progress demo of running CoAP with OSCORE/EDHOC security on RIOT-rs.

## Running

* Run on any board with networking, eg. `laze build -b particlex-xenon run`.
* [Set up networking](../README.md).
* Run `./fauxhoc.py`. This will establish a secure CoAP connection using EDHOC and OSCORE,
  and show some interactions and eventually the log of the device.
* Running multiple concurrent `./fauxhoc.py` instances is supported,
  up to the maximum number of security contexts that are stored (currently 4).

## Roadmap

Eventually, this should be a 20 line demo.

Until the CoAP roadmap is complete,
this serves as a work bench, test bed, demo zone and playground at the same time.
This application will grow as parts of the roadmap are added,
and shrink as they become default or are wrapped into components of RIOT-rs.
