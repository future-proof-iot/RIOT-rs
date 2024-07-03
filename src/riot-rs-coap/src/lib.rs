//! A CoAP stack preconfigured for RIOT-rs.
//!
//! This crate mainly provides easy-to-use wrappers around the [`coapcore`] crate, with presets
//! tailored towards RIOT-rs: It utilizes [`embassy_net`] to open a network accessible CoAP socket
//! and selects [`embedded_nal_coap`] for CoAP over UDP, it selects [`riot_rs_random`] as a source
//! of randomness, and [`lakers_crypto_rustcrypto`] for the cryptographic algorithm
//! implementations.
#![no_std]
#![deny(missing_docs)]
#![deny(clippy::pedantic)]
