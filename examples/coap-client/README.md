# coap client demo

## About

This application sends a CoAP request from the embedded system to a server.

## Warning

On the CoAP client side, security is work in progress and not available for the example yet.

## Running

* [Set up networking](../README.md).
* In one terminal, run `pipx run ./server.py`, which runs the server for the embedded device to interact with.
* Run on any board with networking, eg. `laze build -b particle-xenon run`.

  The board will send a CoAP request saying 'This is Ariel OS',
  and show the server's very loud response.

## Further references

The [server example](../coap-server) is recommended as the first CoAP example,
because it is more suitable for embedded devices, and more mature.

There is a [chapter in the book](https://ariel-os.github.io/ariel-os/dev/docs/book/tooling/coap.html)
that describes more concepts and background.
