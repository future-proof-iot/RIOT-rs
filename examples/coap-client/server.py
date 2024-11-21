#!/usr/bin/env python3
# /// script
# dependencies = [
#   "aiocoap >= 0.4.11, < 0.5",
# ]
# ///
"""
Minimal server providing a single resource /uppercase, to which ASCII text can
be POSTed; the text is returned IN ALL CAPS.
"""

import asyncio
import logging

import aiocoap
from aiocoap.resource import Resource, Site


class Uppercase(Resource):
    async def render_post(self, request):
        text = request.payload.decode("utf8")
        text = text.upper()

        return aiocoap.Message(content_format=0, payload=text.encode("utf8"))


async def main():
    root = Site()

    root.add_resource(["uppercase"], Uppercase())
    await aiocoap.Context.create_server_context(root)
    await asyncio.get_running_loop().create_future()


if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO)
    logging.getLogger("coap-server").setLevel(logging.DEBUG)

    asyncio.run(main())
