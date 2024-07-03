//! A CoAP stack preconfigured for RIOT-rs
//! ======================================
//!
//! This crate mainly provides easy-to-use wrappers around the [`coapcore`] crate, with presets
//! tailored towards RIOT-rs: It utilizes [`embassy-net`] to open a network accessible CoAP socket,
//! [`riot-rs-random`] as a source of randomness, and [`lakers-crypto-rustcrypto`] for the
//! cryptographic algorithm implementations.
#![no_std]
