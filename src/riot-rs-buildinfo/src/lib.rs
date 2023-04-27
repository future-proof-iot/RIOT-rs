//! riot-rs-buildinfo
//!
//! This crate exposes RIOT-rs build information (builder, version, ...).

#![no_std]

include!(concat!(env!("OUT_DIR"), "/buildinfo.rs"));
