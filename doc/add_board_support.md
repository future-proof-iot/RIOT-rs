# Ariel OS adding hardware support

This document serves as a guide as to what's currently needed for adding support
for a board/device to Ariel OS.

Feel free to report anything that's unclear!

## Adding a board

The more similar a board is to one that is already supported, the easier.
It is usually best to copy & adapt an existing one.

- ensure that the architecture [is supported in `riot-rs-embassy`](#adding-an-embassy-architecture)
- in `laze-project.yml`:
  - add `context` for the MCU (if it doesn't exist, yet)
    - parent: the closest Embassy arch
      - selects: a [rustc-target](#adding-a-rustc-target-module) module
      - if the architecture does not have a dedicated SWI, choose one now
        and set `CONFIG_SWI`
      - ensure there's a way to flash the board:
          - if the MCU is supported by probe-rs, add `PROBE_RS_CHIP`
            and `PROBE_RS_PROTOCOL`
          - if the board is based on esp, it should inherit the espflash support
          - else, ask. :)

  - add `builder` for the actual board that uses the context from above as `parent`

Whether to add an intermediate context or just a builder depends on whether the
MCU-specific code can be re-used.

Example for the `st-nucleo-f401re`:

```yaml
contexts:
# ...
- name: stm32f401retx
  parent: stm32
  selects:
    - thumbv7em-none-eabi # actually eabihf, but riot-rs doesn't support hard float yet
  env:
  PROBE_RS_CHIP: STM32F401RETx
  PROBE_RS_PROTOCOL: swd
  RUSTFLAGS:
    - --cfg context=\"stm32f401retx\"
  CARGO_ENV:
    - CONFIG_SWI=USART2

builders:
# ...
- name: st-nucleo-f401re
  parent: stm32f401retx

```

- `src/riot-rs-boards/$BOARD`: add crate that matches board name
  - this crate should inject the board-specific dependencies to the arch crates.
- `src/riot-rs-boards`:
  - `Cargo.toml`: add feature that matches board name
  - `src/lib.rs`: add board to dispatch

## Adding a rustc target module

Each actual rustc target needs its own module in laze-project.yml.
If the device that's being added isn't listed yet, you'll need to take care
of that.

Example:

```yaml
modules:
# ...
  - name: thumbv6m-none-eabi
    depends:
      - cortex-m
    env:
      global:
        RUSTC_TARGET: thumbv6m-none-eabi
        CARGO_TARGET_PREFIX: CARGO_TARGET_THUMBV6M_NONE_EABI
        RUSTFLAGS:
          - --cfg armv6m
```

The variables `RUSTC_TARGET` and `CARGO_TARGET_PREFIX` need to be adjusted.
Add `--cfg $arch` as needed.

Chances are that if you need to add this, you'll also have to add support for the architecture to `riot-rs-threads`.

## Adding an Embassy architecture

As of this writing, Ariel OS supports most architectures that Embassy supports,
including `nrf`, `stm32`, `rp` and `esp-rs`, but excluding `std` and `wasm`.

The steps to add support for another Embassy supported architecture are:

- `src/riot-rs-embassy`:
  - `Cargo.toml`: add Embassy arch dependency
  - `src/lib.rs`: add arch dispatch
  - `src/arch/$ARCH.rs`:
    1. select the appropriate SWI interrupt (if not done through CONFIG_SWI)
    2. implement the `usb` module
