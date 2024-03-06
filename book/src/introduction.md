# Introduction

**RIOT-rs** RIOT-rs is a project aiming to provide a general-purpose 
operating system adequate for low-power Internet of Things (IoT).

RIOT-rs is based on Rust from the ground up, and formal verification for critical modules. 
It combines the awesome Rust embedded ecosystem with RIOT OS.

RIOT-rs builds on top of the hardware abstraction layer 
and async programming framework provided by
[embassy](https://github.com/embassy-rs/embassy) and 
drivers via embedded-HAL.

Aiming to cover versatile use cases, RIOT-rs integrates and combines 
the above HAL with a preemptive scheduler, 
a set of efficient operating system facilities, a bootloader, 
and a curated ecosystem of libraries (available via crates.io)
for cybersecure, memory-safe IoT. 

As a result, a low-power IoT developer can focus on business logic
sitting on top of APIs which remain close to the hardware but
nevertheless stay the same across all supported hardware.
The intent is three-fold: decrease application development time,
increase code portability, and decrease core system vulnerabilities.

RIOT-rs can also be used to host legacy C application and libraries.
However, the essence and ultimate goal of the RIOT-rs is to
provide everything one might need in Rust.

![Architecture](./doc/RIOT-rs-arch-diagram1.svg)
