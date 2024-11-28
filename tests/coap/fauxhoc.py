#!/usr/bin/env -S pipx run
# /// script
# requires-python = ">= 3.10"
# dependencies = [
#   "lakers-python == 0.4.1",
#   "aiocoap[oscore] == 0.4.12",
#   "cbor2",
#   "coap_console == 0.0.3",
# ]
# ///
"""
fake-it-until-you-make-it wrapper for sending an EDHOC+OSCORE request

Its first stage did roughly

~~~
$ ./aiocoap-client 'coap://10.42.0.61/.well-known/edhoc' -m POST --content-format application/cbor-seq --payload "[true, 3, 2, h'0000000000000000000000000000000000000000000000000000000000000000', 0]"
~~~

and the later guessed the C_R and sent a garbled OSCORE request that triggered
the EDHOC CoAP pathways.

Now it does EDHOC using lakers to a point where the name doesn't fit any more…
"""

import asyncio
import random
import argparse

import cbor2
from aiocoap import oscore, Message, POST, Context, GET, error

import lakers

import coap_console

p = argparse.ArgumentParser()
p.add_argument(
    "--random-identity",
    help="Instead of using the known credential, make one up. Chances are the server will not accept this for privileged operations.",
    action="store_true",
)
p.add_argument(
    "--no-demo",
    help="Forego illustrative examples, just start the terminal application",
    action="store_true",
)
p.add_argument(
    "peer",
    help="URI (scheme and host); defaults to the current Ariel OS default {default}",
    default="coap://10.42.0.61",
    nargs="?",
)
args = p.parse_args()

if args.peer.count("/") != 2:
    p.error(
        "Peer should be given as 'coap://[2001:db8::1]' or similar, without trailing slash."
    )

# Someone told us that these are the credentials of devices that are our legitimate peers
eligible_responders_ccs = {
    bytes.fromhex(
        "A2026008A101A5010202410A2001215820BBC34960526EA4D32E940CAD2A234148DDC21791A12AFBCBAC93622046DD44F02258204519E257236B2A0CE2023F0931F1F386CA7AFDA64FCDE0108C224C51EABF6072"
    )
}
eligible_responders = {}  # mapping ID_CRED_R to CRED_R
# when ID_CRED_R is the KID. 8/1/2 is cnf/COSE_Key/kid, IIUC those should be present in suitable CCSs
eligible_responders |= {
    cbor2.dumps({4: bytes.fromhex("0a")}): ccs
    for (parsed, ccs) in ((cbor2.loads(ccs), ccs) for ccs in eligible_responders_ccs)
}
# when ID_CRED_R is CRED_R
eligible_responders |= {ccs: ccs for ccs in eligible_responders_ccs}

if args.random_identity:
    KEY_I, public = lakers.p256_generate_key_pair()
    # For now, that's all Lakers understands; it doesn't even look into -3 (y)
    # b/c it doesn't need it for key derivation, which is fortunate because the
    # generator doesn't produce one either. (It's not like this key is going to
    # be used for signing or encryption).
    cred_i_data = {2: "me", 8: {1: {1: 2, 2: b"\x2b", -1: 1, -2: public, -3: b"0"}}}
    # We could slim it down to
    # >>> cred_i_data = {8: {1: {1: 2, -1: 1, -2: public}}}
    # but even if the peer had the code to process that into a valid
    # credential, the Lakers Python API currently doesn't allow creating
    # credential through any other code path than through CredentialRPK::parse.
    CRED_I = cbor2.dumps(cred_i_data)
    cred_i_mode = lakers.CredentialTransfer.ByValue
else:
    # Those are currently hardcoded, will late be configurable, and ultimately not needed if using ACE
    CRED_I = bytes.fromhex(
        "A2027734322D35302D33312D46462D45462D33372D33322D333908A101A5010202412B2001215820AC75E9ECE3E50BFC8ED60399889522405C47BF16DF96660A41298CB4307F7EB62258206E5DE611388A4B8A8211334AC7D37ECB52A387D257E6DB3C2A93DF21FF3AFFC8"
    )
    KEY_I = bytes.fromhex(
        "fb13adeb6518cee5f88417660841142e830a81fe334380a953406a1305e8706b"
    )
    # Because the peer knows, but also because it's just a bit too long to pass around by value
    cred_i_mode = lakers.CredentialTransfer.ByReference


class EdhocSecurityContext(
    oscore.CanProtect, oscore.CanUnprotect, oscore.SecurityContextUtils
):
    def __init__(self, initiator, c_ours, c_theirs):
        # initiator could also be responder, and only this line would need to change
        # FIXME Only ByReference implemented in edhoc.rs so far
        self.message_3, _i_prk_out = initiator.prepare_message_3(cred_i_mode, None)

        if initiator.selected_cipher_suite() == 2:
            self.alg_aead = oscore.algorithms["AES-CCM-16-64-128"]
            self.hashfun = oscore.hashfunctions["sha256"]
        else:
            raise RuntimeError("Unknown suite")

        # we check critical EADs, no out-of-band agreement, so 8 it is
        oscore_salt_length = 8
        # I figure that one would be ageed out-of-band as well
        self.id_context = None
        self.recipient_replay_window = oscore.ReplayWindow(32, lambda: None)

        master_secret = initiator.edhoc_exporter(0, [], self.alg_aead.key_bytes)
        master_salt = initiator.edhoc_exporter(1, [], oscore_salt_length)
        print(f"Derived {master_secret=} {master_salt=}")

        self.sender_id = c_theirs
        self.recipient_id = c_ours
        if self.sender_id == self.recipient_id:
            raise ValueError("Bad IDs: identical ones were picked")

        self.derive_keys(master_salt, master_secret)

        self.sender_sequence_number = 0
        self.recipient_replay_window.initialize_empty()

    def post_seqnoincrease(self):
        pass

    def protect(self, message, request_id=None, *, kid_context=True):
        outer_message, request_id = super().protect(
            message, request_id=request_id, kid_context=kid_context
        )
        if self.message_3 is not None:
            outer_message.opt.edhoc = True
            outer_message.payload = self.message_3 + outer_message.payload
            self.message_3 = None
        return outer_message, request_id


async def main():
    ctx = await Context.create_client_context()

    # We only run one connection so we don't care, but let's spread it
    c_i = bytes([random.randint(0, 23)])
    initiator = lakers.EdhocInitiator()
    message_1 = initiator.prepare_message_1(c_i)

    print(f"Initiating EDHOC session with the device at {args.peer}")

    msg1 = Message(
        code=POST,
        uri=args.peer + "/.well-known/edhoc",
        payload=cbor2.dumps(True) + message_1,
        # payload=b"".join(cbor2.dumps(x) for x in [True, 3, 2, b'\0' * 32, 0]),
    )
    msg2 = await ctx.request(msg1).response_raising

    (c_r, id_cred_r, ead_2) = initiator.parse_message_2(msg2.payload)

    cred_r = eligible_responders[id_cred_r]
    initiator.verify_message_2(
        KEY_I, CRED_I, cred_r
    )  # odd that we provide that here rather than in the next function

    print(
        f"EDHOC setup in progress. Message 2 has been received, and we verified that the device is who we expect it to be. Sending actual requests now, along with the final message 3 that tells the device that we are authorized to do that."
    )

    oscore_context = EdhocSecurityContext(initiator, c_i, c_r)

    ctx.client_credentials[args.peer + "/*"] = oscore_context

    if not args.no_demo:
        msg3 = Message(
            code=GET,
            uri=args.peer + "/.well-known/core",
        )

        wkc = (await ctx.request(msg3).response_raising).payload.decode("utf8")
        print(
            "Success: As a response was received, we know that encrypted communication was established."
        )
        print()
        print("Received /.well-known/core (discovery information):")
        print(wkc)
        print()

        normalrequest = Message(
            code=GET,
            uri=args.peer + "/poem",
        )
        poem = (await ctx.request(normalrequest).response_raising).payload.decode(
            "utf8"
        )
        print("Received /poem (a resource transported in multiple blocks):")
        print(poem)
        print()

    print(
        f"Requesting additional diagnostic data (success is {'not expected' if args.random_identity else 'expected'})"
    )
    try:
        # pre-flight b/c read_stream_to_console has bad error reporting
        await ctx.request(Message(code=GET, uri=args.peer + "/stdout")).response_raising
    except error.ResponseWrappingError as e:
        print(
            "Received encrypted response but no success:",
            e.coapmessage.code,
            e.coapmessage.payload.decode("utf8"),
        )
    else:
        print(
            "The remaining output is diagnostic output from the device, and will be updated continuously:"
        )
        await coap_console.read_stream_to_console(ctx, args.peer + "/stdout")

    await ctx.shutdown()


if __name__ == "__main__":
    asyncio.run(coap_console.aiocoap_errors_are_pretty(main()))
