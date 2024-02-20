# RIOT-rs
[![Build Status][build-badge]][build-info]
[![Documentation][doc-badge]][documentation-mdbook]
[![Matrix][matrix-badge]][matrix-link]

> Rust & RIOT combined for ergonomic embedded development

This is an experimental project to provide a nice base OS for embedded
development on low-end IoT devices (with some kilobytes of RAM/flash, think
Cortex-M). It combines the awesome Rust embedded ecosystem with RIOT.

**This is highly experimental. Expect heavy changes and breakage!**

If you're looking for a more production ready way of writing RIOT applications
in Rust, check out [riot-wrappers](https://gitlab.com/etonomy/riot-wrappers).

## Goals

- improve RIOT using the merits of Rust.
- provide a "rusty" development workflow (e.g., using cargo / crates.io)
- provide a nice Rust API, framework and collection of crates suitable for
  embedded development
- rewrite parts of RIOT in Rust to improve robustness and maintainability

## Supported hardware

The following list of hardware is currently supported
 - [Nordic nRF52840 DK](https://www.nordicsemi.com/Products/Development-hardware/nRF52840-DK)
 - [Raspberry Pi Pico](https://www.raspberrypi.com/products/raspberry-pi-pico/)

## Status

The current iteration of RIOT-rs combines [embassy](https://embassy.dev/) with
a preemptive scheduler and adds some integration and build system work.

## Quickstart

Assuming you have a Nordic nrf52840dk connected, the following guidelines
provides instructions for flashing and running the [`hello-world`
example](https://github.com/future-proof-iot/RIOT-rs/tree/main/examples/hello-world):

### Prerequisites

1.install needed system dependencies. On Ubuntu, the following is sufficient:

        apt install build-essential curl git python3 pkg-config \
                   libssl-dev llvm-dev cmake libclang-dev gcc-arm-none-eabi \
                   clang libnewlib-nano-arm-none-eabi unzip lld ninja-build

1. install [rustup](https://rustup.rs/)

1. install [laze](https://github.com/kaspar030/laze): `cargo install laze`

1. install [probe-rs](https://github.com/probe-rs/probe-rs): `cargo install probe-rs --features cli`
   (2023-10-17: if that fails, try from git: `cargo install --git https://github.com/probe-rs/probe-rs --features cli probe-rs`)

1. clone this repository and cd into it

1. install rust targets: `laze build install-toolchain`

### Run the example

1. Compile, flash and the hello-world example using `probe-rs run`

        laze -C examples/hello-world build -b nrf52840dk -s probe-rs-run run

## More information

Please look [at the build system documentation](doc/build_system) for more usage
information.

## Minimum Supported Rust Version (MSRV)

RIOT-rs makes heavy use of Rust unstable features. For the time being, it is
recommended to use a current nightly.

## Coding Conventions

Please see the chapter on
[coding conventions](https://future-proof-iot.github.io/RIOT-rs/dev/coding-conventions.html)
in the documentation.

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

[build-badge]: https://github.com/future-proof-iot/RIOT-rs/actions/workflows/main.yml/badge.svg
[build-info]: https://github.com/future-proof-iot/RIOT-rs/actions/workflows/main.yml
[matrix-badge]: https://img.shields.io/badge/chat-Matrix-brightgreen.svg
[matrix-link]: https://matrix.to/#/#RIOT-rs:matrix.org
[doc-badge]: https://img.shields.io/badge/Documentation-%F0%9F%93%94-blue
[documentation-mdbook]: https://future-proof-iot.github.io/RIOT-rs/dev/
