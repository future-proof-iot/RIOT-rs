# CoAP

> **Warning**: This documentation is currently ahead of the implementation roadmap.

[CoAP] is an **application layer protocol similar to HTTP** in its use
(eg. you could POST data onto a resource on a server that is identified by a URL)
but geared towards IoT devices in its format and its security mechanisms.

CoAP provides a versatile set of transports
(with IETF Proposed Standards for running [over UDP] including multicast, [over TCP and WebSockets],
other standards for running [over SMS and NB-IoT], and more in development).
It relies on proxies to span across transports and to accommodate the characteristics of particular networks,
and offers features exceeding the classical REST set such as [observation].

**RIOT-rs supports** the use of CoAP
for implementing clients, servers or both in a single device.
As part of our mission for strong security,
we use encrypted CoAP traffic by default as explained below.

[CoAP]: https://coap.space/
[over UDP]: https://datatracker.ietf.org/doc/html/rfc7252
[over TCP and WebSockets]: https://datatracker.ietf.org/doc/html/rfc8323
[over SMS and NB-IoT]: https://www.omaspecworks.org/wp-content/uploads/2018/10/Whitepaper-11.1.18.pdf
[observation]: https://datatracker.ietf.org/doc/html/rfc7641

## Usage: Server side

An example of a CoAP server is [provided as `examples/coap`], see [its `run()` function] for the practical steps.

A CoAP server is created by assembling several **resource handlers on dedicated paths**:
There might be a path `/s/0` representing a particular sensor,
and a path `/fwup` for interacting with firmware updates.
The CoAP implementation can put additional resources at well-known locations,
eg. `/.well-known/core` for discovery or `/.well-known/edhoc` for establishing secure connections.

The handler needs to concern itself with security aspects of the request content
(eg. file format parsers should treat incoming data as possibly malformed),
but the decision whether or not a request is allowed is delegated to an [access policy](#server-access-policy).

[provided as `examples/coap`]: https://github.com/ariel-os/ariel-os/tree/main/examples/coap
[its `run()` function]: https://github.com/ariel-os/ariel-os/blob/2b76e560394884d3c8f7eaae51beefd59a316d7b/examples/coap/src/main.rs#L70


## Usage: Client side

The example [provided as `examples/coap`] also contains client steps.

A program that triggers a CoAP request provides[^whatsinarequest] some components to the CoAP stack before phrasing the actual request:

* A **URL describing the resource**, eg. `coap://coap.summit.riot-os.org/agenda` or `coap+tcp://[2001:db8::1]/.well-known/core`.

  Note that while the address is printed here as a text URL
  (and may even be entered as such in code),
  its memory and transmitted representations are binary components.

* Optionally, directions regarding how to reach or find the host.

  This is not needed when there are IP literals involved, or DNS is available to resolve names.
  Directions may be available if the application discovered a usable proxy,
  or when it is desired to use opt-in discovery mechanisms such as multicast.

* A policy reference on how to authenticate the server, and which identity to present.
  This is optional if there is a global policy,
  or if there is an implied security mechanism for the URL.


[^whatsinarequest]: The components required for a request are not documented as such in the CoAP RFCs,
    but it is the author's opinion that they are a factual requirement:
    Implementations may implicitly make decisions on those,
    but the decisions are still made.
    At the time of writing, there [is an open issue](https://github.com/core-wg/corrclar/issues/41) to clarify this in the specifications.

## Security

The CoAP stack is configured with server and client policies.
The security mechanisms used depend on those selected in the policies.

At this stage, RIOT-rs uses three pieces of security components:
OSCORE (for symmetric encryption), EDHOC (for key exchange) and ACE (for authentication).

OSCORE/EDHOC/ACE were chosen first because they scale down
well to the smallest devices, and because they
all have in common that they sit naturally on top of CoAP:
Their communication consists of CoAP requests and responses.
Thus, they work homogeneously across all CoAP transports,
and provide end-to-end security across untrusted proxies.

Alternatives are possible (for instance DTLS, TLS, IPsec or link-layer encryption)
but are currently not implemented / not yet supported in RIOT-rs.

### Server access policy

A policy is configured for the whole server depending on the desired security mechanisms.
Examples of described policy entries are:

* This is a fixed public key, and requests authenticated with that key are allowed to GET and PUT to `/limit`.
* The device has a shared secret from its authorization server, with which the authorization server secures the tokens it issues to clients. Clients may perform any action as long as they securely present a token that allows it. For example, a token may allow GET on `/limit` and PUT on `/led/0`.
* Any (even unauthenticated) device may GET `/hello/`.

#### Interacting with a RIOT-rs CoAP server from the host

A convenient policy (which is the default of RIOT-rs's examples)
is to grant the user who flashes the device all access on it.
When that policy is enabled in the build system,
an unencrypted key is created in the developer's [state home directory]<!-- precise location TBD -->,
from where it can be picked up by tools such as [aiocoap-client].

Furthermore,
when a CoAP server is provisioned through the RIOT-rs build system,
public keys and their device associations are stored
in the developer's state home directory.

Together, these files act in a similar way as the classic UNIX files `~/.netrc` and `~/.ssh/id_rsa{,.pub}`.
They can also double as templates for an application akin to `ssh-copy-id`
in that they enable a server policy like
"any device previously flashed on this machine may GET all resources".

[aiocoap-client]: https://aiocoap.readthedocs.io/en/latest/tools.html
[state home directory]: https://specifications.freedesktop.org/basedir-spec/latest/

### Client policies

The policy for outgoing requests can be defined globally or per request.

Examples of policies that can be available are
  "expect the server to present some concrete public key, use this secret key once the server is verified",
  "use a token for this audience and scope obtained from that authentication server",
  "expect the server to present a chain of certificates for its hostname down to a set of root certificates" (which is the default for web browsers),
  "establish an encrypted connection and trust the peer's key on first use",
  down to "do not use any encryption".

### Available security mechanisms

These components are optional, but enabled by default --
when all are disabled, the only sensible policy that is left <!-- "deny everything" is not sensible, could just not include CoAP then -->
is to allow unauthenticated access everywhere.
For example, this may make sense on a link layer with tight access control.
The components also have internal dependencies:
EDHOC can only practically be used in combination with OSCORE;
ACE comes with profiles with individual dependencies
(eg. using the ACE-EDHOC profile requires EDHOC).

#### Symmetric encryption: OSCORE

OSCORE ([RFC8613]) provides symmetric encryption for CoAP requests:
It allows clients to phrase their CoAP request,
encrypts the parts that a proxy does not need to be aware of[^parts]
and sends on the ciphertext in a CoAP request.
The server can decrypt the request,
process it like any other request (but with the access policy evaluated according to the key's properties),
and its response gets encrypted likewise.

Working with symmetric keys requires a lot of care and effort managing keys:
assigning the same key twice can have catastrophic consequences,
and even recovering from an unplanned reboot is by far not trivial.

RIOT-rs does not offer direct access to OSCORE for those reasons,
and uses OSCORE's companion mechanisms to set up keys.

Policies are not described in terms of OSCORE keys.

[RFC8613]: https://datatracker.ietf.org/doc/html/rfc8613
[^parts]: Most of the message is encrypted.
  Noteworthy unencrypted parts are the hostname the request is sent to,
  the parts that link a response to a request,
  and housekeeping details such as whether a request is for an observation and thus needs to be kept alive longer.

#### Key establishment: EDHOC

EDHOC ([RFC9528]) is a key establishment protocol
that uses asymmetric keys to obtain mutual authentication and forward secrecy.
In particular, two CoAP devices can run EDHOC over CoAP to obtain key material for OSCORE,
which can then be used for fast communication.

Unless ACE or certificate chains are used,
the main use of EDHOC in RIOT-rs is with raw public keys:
Devices (including the host machine) generate a private key,
make the corresponding public key known,
and then send the public key (or its short ID) along with EDHOC messages to be identified by that public key.
This is similar to how SSH keys are commonly used.

Policies described in terms of EDHOC keys include the public key of the peer,
which private key to use,
and whether our public key needs to be sent as a full public key or can be sent by key ID.

[RFC9528]: https://datatracker.ietf.org/doc/html/rfc9528

#### Authorization: ACE

The ACE framework ([RFC9200]) describes how a trusted service (the Authorization Server, "AS")
can facilitate secure connections between devices that are not explicitly configured to be used together.
It frequently issues CWTs (CBOR Web Tokens) that are to JWTs (JSON Web Tokens) as CoAP is to HTTP.

The general theme of it is that
a client that wants to access some resource on a server under the AS's control
first asks the AS for an access token,
and then presents that token to the server (called Resource Server / RS in that context).

Clients know the AS by having means to establish some secure connection with it,
eg. through EDHOC.
Resource Servers know the AS by verifying the tokens they receive from the AS,
eg. by having a shared secret with the AS or knowing its signing key.

Details of the process are specified in ACE profiles;
apart from those listed here, popular profiles include the DTLS profile and the profiles for group communication.

[RFC9200]: https://datatracker.ietf.org/doc/html/rfc9200

##### ACE-OSCORE profile

With the ACE-OSCORE profile ([RFC9203]),
the AS provides a random OSCORE key in the token (which is encrypted for the RS),
and sends the token along with the same OSCORE key through its secure connection to the client.
Before the client can send OSCORE requests,
it POSTs the token to the server over an unprotected connection
(the token itself is encrypted),
along with a random number and some housekeeping data
that go into the establishment of an OSCORE context.

The [documentation of the CoAP/ACE-OAuth PoC project] describes the whole setup in more detail.

[documentation of the CoAP/ACE-OAuth PoC project]: https://gitlab.com/oscore/coap-ace-poc-overview

[RFC9203]: https://datatracker.ietf.org/doc/html/rfc9203

##### ACE-EDHOC profile

The ACE-EDHOC profile is [under development in the ACE working group].

It differs from the ACE-OSCORE profile
in that the AS does not provide symmetric key material
but only points out the respective peer's public keys.

The token is not sent in plain,
but as part of the EDHOC exchange.

[under development in the ACE working group]: https://datatracker.ietf.org/doc/draft-ietf-ace-edhoc-oscore-profile/

##### Using ACE from the host during development

While full operation of ACE requires having an AS as part of the network,
CoAP servers running on RIOT-rs can be used in the ACE framework without a live server.

Similar to how an EDHOC key is created on demand on the host,
an AS's list of Resource Servers is maintained by default.
Tools at the host can then use the locally stored key to create tokens that grant fine-grained permissions on the RIOT-rs device.

With the [New ACE Workflow developed in ACE],
such tokens can also be provisioned into Resource Servers on behalf of clients that are being provisioned.
Thus,
the offline AS can enable deployed RIOT-rs based CoAP servers
to accept requests from newly created RIOT-rs based CoAP clients
without the need for the CoAP client to create a network connection to the host.
(Instead, the host needs to find the Resource Server over the network).

.. note: Some more exploration of this workflow will be necessary
  as to how the client can trigger the AS to re-install (or renew) its token
  in case the Resource Server retired the token before its expiration.
  For RIOT-rs internal use,
  AS-provisioned tokens might just be retained longer.

[New ACE Workflow developed in ACE]: https://www.ietf.org/archive/id/draft-ietf-ace-workflow-and-params-00.html#name-new-ace-workflow
