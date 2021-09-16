[![Build Status](https://drone.schleiser.de/api/badges/future-proof-iot/RIOT-rs/status.svg?ref=refs/heads/main)](https://drone.schleiser.de/future-proof-iot/RIOT-rs)

# RIOT-rs

    Rust & RIOT OS combined for ergonomic embedded development

This is an experimental project trying to provide a nice base OS for embedded
development on low-end IoT devices (with some kilobytes of RAM/flash, think Cortex-M).
It combines the awesome Rust embedded ecosystem with RIOT OS.

**This is highly experimental. Expect heavy changes and breakage!**

If you're looking for a more production ready way of writing RIOT applications
in Rust, check out [riot-wrappers](https://gitlab.com/etonomy/riot-wrappers).

## Supported hardware

This currently only supports the Nordic nrf52840dk.

## Goals

- improve RIOT OS using the merits of Rust.
- provide a "rusty" development workflow (e.g., using cargo / crates.io)
- provide a nice Rust API, framework and collection of crates suitable for
  embedded development
- rewrite parts of RIOT in Rust to improve robustness and maintainability

## Status

This project currently allows to build a RIOT application with an embedded Rust
based rt+linking. In addition, it provides a Rust rewrite of RIOT's "core" module,
providing API compatible implementations of RIOT's scheduler, mutexes, thread flags,
msg and mbox.
[riot-wrappers](https://gitlab.com/etonomy/riot-wrappers) are integrated for
developing applications using Rust, though this has not been used / tested much.

## Quickstart

Assuming you have a Nordic nrf52840dk connected, this should get you somewhere:

### Prerequisites

1.install needed system dependencies. On Ubuntu, this should be sufficient:

        apt-get install build-essential curl git python3 pkg-config \
                   libssl-dev llvm-dev cmake libclang-dev gcc-arm-none-eabi \
                   clang libnewlib-nano-arm-none-eabi unzip lld ninja-build

1. install [rustup](https://rustup.rs/)

1. install [laze](https://github.com/kaspar030/laze): `cargo install laze`

1. clone this repository and cd into it

1. set up RIOT-rs toolchain

        laze task install-toolchain
        laze task install-c2rust
        laze task clone-riot

### Flash some example

1. Compile and flash the bottles example

   laze -C examples/bottles task -b nrf52840dk flash

1. in another window, run a terminal program of your choice to watch the
  nrf52840dk's serial output

## Building RIOT C applications with RIOT-rs

It is possible to build any RIOT application using RIOT-rs and its core
implementation, using `examples/riot-app`:

    laze -Cexamples/riot-app task -b nrf52840dk -a riot-app -DRIOT_APP=foo/bar flash

See the [riot-app docs](examples/riot-app/README.md) for more information.

## More information

Please look [here](doc/build_system.md) for more usage information.

## Minimum Supported Rust Version (MSRV)

RIOT-rs makes heavy use of Rust unstable features. For the time being, it is
recommended to use a current nightly.

## Copyright & License

RIOT-rs is licensed under either of

- Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

at your option.

RIOT-rs links with many components of [RIOT OS](https://github.com/RIOT-OS/RIOT),
which is licenced under the terms of LGPLv2.1.

Copyright (C) 2020 Freie Universität Berlin, Inria, Kaspar Schleiser

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
