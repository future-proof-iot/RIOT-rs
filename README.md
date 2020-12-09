# RIOT-rs

    Rust & RIOT OS combined for ergonomic embedded development

This is an experimental project trying to provide a nice base OS for embedded
development on low-end IoT devices (with some kilobytes of RAM/flash, think Cortex-M).
It combines the awesome Rust embedded ecosystem with RIOT OS.

__This is highly experimental. Expect heavy changes and breakage!__

If you're looking for a more production ready way of writing RIOT applications
in Rust, check out [riot-wrappers](https://gitlab.com/etonomy/riot-wrappers).

## Supported hardware

This currently only supports the Nordic nrf52840dk.

## Goals

- improve RIOT OS using the merits of Rust.
- provide a "rusty" development workflow (e.g., using cargo / crates.io)
- provide a nice Rust API, framework and collection of crates suitable for embedded development
- rewrite parts of RIOT in Rust to improve robustness and maintainability

## Quickstart

There's currently one example that will run on one board. Assuming you have
a Nordic nrf52840dk connected, this should get you somewhere:

1. install needed system dependencies. On Ubuntu, this should be sufficient:

        $ apt-get install build-essentials curl git python3 pkg-config libssl-dev llvm-dev cmake libclang-dev gcc-arm-none-eabi clang libnewlib-nano-arm-none-eabi unzip

1. install [rustup](https://rustup.rs/)
1. install [just](https://github.com/casey/just): `cargo install just`
1. clone this repository and cd into it
1. run `just install-reqs` (Warning: this will also compile c2rust, which needs *a lot* (>6gb) of RAM)
   this will require the dev packages for some system libraries: TBD
1. run `just bin=examples/bottles run`. in gdb, press "c" followed by enter
1. in another window, run a terminal program of your choice to watch the
   nrf52840dk's serial output

## Minimum Supported Rust Version (MSRV)

RIOT-rs makes heavy use of Rust unstable features. For the time being, it is
recommended to use a current nightly.

## License

RIOT-rs is licensed under either of

- Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

at your option.

RIOT-rs links with many components of [RIOT OS](https://github.com/RIOT-OS/RIOT),
which is licenced under the terms of LGPLv2.1.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
