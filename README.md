# RIOT-rs
[![Build Status][build-badge]][build-info]
[![Book][book-badge]][documentation-mdbook]
[![Documentation][rustdoc-badge]][documentation-dev-rustdoc]
[![Matrix][matrix-badge]][matrix-link]

> Rust & RIOT combined for ergonomic embedded development

RIOT-rs is an operating system for secure, memory-safe, low-power Internet of Things (IoT).
RIOT-rs is based on Rust from the ground up, and uses formal verification
for critical modules. To learn more about our motivations, see this
[manifesto](https://ariel-os.github.io/ariel-os/dev/docs/book/manifesto.html).

Hardware targets include varieties of IoT hardware based on 
32-bit microcontroller architectures (such as Cortex-M, RISC-V).

In practice, RIOT-rs builds on top of [Embassy](https://github.com/embassy-rs/embassy).
Compared to what [Embassy](https://github.com/embassy-rs/embassy) already provides,
RIOT-rs brings additional value in terms of 
abstraction, operating system functionalities, 
and integration for a (curated) set of software modules, tools and libraries, as well as 
a stronger focus on cybersecurity and formal verification.
 
In particular, RIOT-rs aims to combine:

- **application code portability** across all supported hardware, via consistent memory/energy efficient APIs;
- **async programming** paradigms, based on [Embassy](https://github.com/embassy-rs/embassy);
- **preemptive scheduler** programming paradigms, based on formally verified modules using [hax](https://hacspec.org/blog/posts/hax-v0-1/);
- **booting & update security**, via measured boot and secure software updates, using formally verified modules.

Overall, RIOT-rs gives you a 'batteries-included' experience, on par
with [RIOT](https://github.com/RIOT-OS/RIOT). 

## Supported hardware

The following list of hardware is currently supported:
 - [Nordic nRF52840 DK](https://www.nordicsemi.com/Products/Development-hardware/nRF52840-DK) (Cortex-M4)
 - [Nordic nRF5340 DK](https://www.nordicsemi.com/Products/Development-hardware/nRF5340-DK) (Cortex-M33)
 - [Raspberry Pi Pico](https://www.raspberrypi.com/products/raspberry-pi-pico/) (RP2040, Cortex-M0+)
 - [Raspberry Pi Pico W](https://www.raspberrypi.com/products/raspberry-pi-pico/) (RP2040, Cortex-M0+)
 - [BBC Micro:Bit v2](https://tech.microbit.org/hardware/2-0-revision/) (Cortex-M4)
 - [Expressif ESP32-C6-DevKitC-1](https://docs.espressif.com/projects/espressif-esp-dev-kits/en/latest/esp32c6/esp32-c6-devkitc-1/user_guide.html) (RISC-V)
 - [ST NUCLEO-F401RE](https://www.st.com/en/evaluation-tools/nucleo-f401re.html) (Cortex-M4)
 - [ST NUCLEO-H755ZI-Q](https://www.st.com/en/evaluation-tools/nucleo-h755zi-q.html) (Cortex-M7)
 - [ST NUCLEO-WB55RG](https://www.st.com/en/evaluation-tools/p-nucleo-wb55.html) (Cortex-M4)
 - and more to come soon.

## Status

**This is currently work-in-progress. Expect missing functionalities and frequent changes!** 
If you are not so adventurous, but nevertheless looking for a way 
to run your Rust module on a microcontroller, you could try to 
glue it directly on top of [Embassy](https://github.com/embassy-rs/embassy), 
or instead, run your module in a [riot-wrappers](https://github.com/RIOT-OS/rust-riot-wrappers).

## Quickstart

The following assumes you have a Nordic nrf52840dk connected to your PC.
(For other supported boards, you can find your board's name in
[./src/riot-rs-boards/Cargo.toml](https://github.com/ariel-os/ariel-os/blob/main/src/riot-rs-boards/Cargo.toml)
and use it instead of 'nrf52840dk' in the below guidelines.)

The following instructions will enable you to flash and run the [`hello-world`
example](https://github.com/ariel-os/ariel-os/tree/main/examples/hello-world):

### Prerequisites

1. install needed system dependencies. On Ubuntu, the following is sufficient:

        apt install build-essential curl git python3 pkg-config \
                   libssl-dev llvm-dev cmake libclang-dev gcc-arm-none-eabi \
                   clang libnewlib-nano-arm-none-eabi unzip lld ninja-build

1. install [rustup](https://rustup.rs/)

1. install [laze](https://github.com/kaspar030/laze): `cargo install laze`

1. install [probe-rs](https://github.com/probe-rs/probe-rs): `cargo install probe-rs-tools --locked`

1. clone this repository and cd into it

1. install rust targets: `laze build install-toolchain`

### Run the example

1. Compile, flash and the hello-world example using `probe-rs run`

        laze -C examples/hello-world build -b nrf52840dk run

![Example](./doc/hello-world_render.svg)

<details>
<summary> (might fail if the flash is locked, click here for unlocking instructions) </summary>
This might fail due to a locked chip, e.g., on most nrf52840dk boards that are fresh from the factory.
In that case, the above command throws an error that ends with something like this:

```
An operation could not be performed because it lacked the permission to do so: erase_all
```

The chip can be unlocked using this command:

    laze -C examples/hello-world build -b nrf52840dk flash-erase-all
</details>

## More information

Please look [at the build system documentation](doc/build_system.md) for more usage
information.

## Minimum Supported Rust Version (MSRV)

RIOT-rs makes heavy use of Rust unstable features. For the time being, it is
recommended to use a current nightly.

## Coding Conventions

Please see the chapter on
[coding conventions](https://ariel-os.github.io/ariel-os/dev/docs/book/coding-conventions.html)
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

[build-badge]: https://github.com/ariel-os/ariel-os/actions/workflows/main.yml/badge.svg
[build-info]: https://github.com/ariel-os/ariel-os/actions/workflows/main.yml
[matrix-badge]: https://img.shields.io/badge/chat-Matrix-brightgreen.svg
[matrix-link]: https://matrix.to/#/#ariel-os:matrix.org
[book-badge]: https://img.shields.io/badge/Book-%F0%9F%93%94-blue
[rustdoc-badge]: https://img.shields.io/badge/Documentation-%F0%9F%93%94-blue
[documentation-mdbook]: https://ariel-os.github.io/ariel-os/dev/docs/book/
[documentation-dev-rustdoc]: https://ariel-os.github.io/ariel-os/dev/docs/api/
