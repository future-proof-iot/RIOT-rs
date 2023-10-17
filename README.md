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

The current iteration of RIOT-rs combines [embassy](https://embassy.dev/) with
a preemptive scheduler and adds some integration and build system work.
It's highly experimental, expect frequent changes and breakage.

## Quickstart

Assuming you have a Nordic nrf52840dk connected, this should get you somewhere:

### Prerequisites

1.install needed system dependencies. On Ubuntu, this should be sufficient:

        apt-get install build-essential curl git python3 pkg-config \
                   libssl-dev llvm-dev cmake libclang-dev gcc-arm-none-eabi \
                   clang libnewlib-nano-arm-none-eabi unzip lld ninja-build

1. install [rustup](https://rustup.rs/)

1. install [laze](https://github.com/kaspar030/laze): `cargo install laze`

1. install [probe-rs](https://github.com/probe-rs/probe-rs): `cargo install probe-rs --features cli`
   (2023-10-17: if that fails, try from git: `cargo install --git https://github.com/probe-rs/probe-rs --features cli`)

1. clone this repository and cd into it

### Run some example

1. Compile, flash and the hello-world example using `probe-rs run`

        laze -C examples/hello-world build -b nrf52840dk -s probe-rs-run run

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

~~RIOT-rs links with many components of [RIOT OS](https://github.com/RIOT-OS/RIOT),
which is licenced under the terms of LGPLv2.1.~~

Copyright (C) 2020-2023 Freie Universität Berlin, Inria, Kaspar Schleiser

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
