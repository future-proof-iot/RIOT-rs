# About

This is embedded-threads, an experimental embedded Rust scheduler.

A while ago, I re-wrote the RIOT scheduler in Rust. That was supposed to be
API- compatible and run RIOT applications as-is. The result worked and was of
comparable performance, but the implementation turned out too C-like.

So this is a try to create a scheduler that is sound, but still allows to
implement RIOT's semantics "on top".

I hope to make this scheduler flexible and usable enough to be usable outside
RIOT-rs context.

## Status

Basic threading is implemented, for Cortex-M3+ (thumbv7m). This is
experimental, expect changes and breakage. Feedback welcome!

## Roadmap

- [x] basic thread switching
  - [ ] Cortex-M
    - [x] Cortex-M3+
    - [ ] Cortex-M0+
    - [ ] floating-point support
  - [ ] RISCV
  - [ ] ESP32 (Xtensa)
- [ ] MPU support / sandboxing
- [x] highest-runnable-first scheduler
- [ ] provide basic IPC
  - [x] locks/mutexes
  - [x] thread flags
  - [ ] channels
- [ ] integrate into [RIOT-rs](https://github.com/future-proof-iot/RIOT-rs)
- [ ] make scheduler implementation pluggable
- [ ] provide formally verified implementation
- [ ] provide best-in-class performance

## Licence

embedded-threads is licensed under the terms of the Apache license (version 2.0).
