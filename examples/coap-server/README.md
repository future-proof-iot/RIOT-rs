# coap server demo

## About

This application starts a minimal CoAP server.

The server offers a single resource, `/hello`, which returns a friendly message.

The default policy allows access to the resource,
but clients can cryptographically verify that they are talking to the right server using its public key.
Both the policy and the key are currently hard-coded;
making the former configurable and the latter dynamic is work in progress.

## Running

* Run on any board with networking, eg. `laze build -b particle-xenon run`.
* [Set up networking](../README.md).
* Run `aiocoap-client`
  to list the resources of the device:

  ```sh
  $ pipx install 'aiocoap[all]'
  $ aiocoap-client coap://10.42.0.61/.well-known/core --credentials client.diag
  # application/link-format content was re-formatted
  </hello>
  ```

  If you prefer not to install the CoAP client, you can
  replace any call to `aiocoap-client` with `pipx run --spec 'aiocoap[all]' aiocoap-client` instead.

  The output tells you there is a `/hello` resource, so read that next:

  ```sh
  $ aiocoap-client coap://10.42.0.61/hello --credentials client.diag
  Hello from Ariel OS
  ```

  The argument `--credentials client.diag` tells the client to establish a secure connection.
  Without the argument, the requests come through just as well,
  but the client has no assurance on the server's identity.

## Further references

There is a [chapter in the book](https://ariel-os.github.io/ariel-os/dev/docs/book/tooling/coap.html)
that describes more concepts and background.
