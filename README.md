riot-rs-runqueue
================

This repository contains the Runqueue as used by RIOT-rs.
It basically encodes the scheduling, as in, "which thread should be switched to next?".

How to use
----------

The crate is not supposed to be used on its own, but as dependendy of
[RIOT-rs](https://github.com/future-proof-iot/RIOT-rs).

Code layout
-----------

`lib.rs` contains the public API. `runqueue.rs` contains
the only current implementation.

We expect other implementations to show up (with different trade-offs), which
can hopefully switched using crate features. For that reason, there are some
tests against the public API in `lib.rs`.

Minimum Supported Rust Version (MSRV)
-------------------------------------

This crate currently requires a recent compiler supporting const fn.
For the time being, it is recommended to use a current nightly.

Copyright & License
-------------------

riot-rs-runqueue is licensed under either of

    Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
    MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

at your option.

Copyright (C) 2021 Freie Universität Berlin, Inria, Kaspar Schleiser

Contributing
------------

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
