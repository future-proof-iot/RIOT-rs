//! A CoAP stack for embedded devices with built-in OSCORE/EDHOC support
//! ====================================================================
//!
//! This crate provides an asynchronous task that serves CoAP requests on a UDP port provided by
//! the application as an `embedded-nal` socket, and processes CoAP along with its security
//! components OSCORE and EDHOC before passing on authorized requests to the application.
//!
//! The crate is under heavy development: Its API is in flux, and so far it does not yet provide
//! the CoAP server itself, but merely a middleware. (Providing the full CoAP will be a requirement
//! for at least as long as the OSCORE component is tightly coupled to a particular implementation
//! of [`coap-message`]).
#![no_std]
#![feature(lint_reasons)]
#![expect(
    clippy::indexing_slicing,
    reason = "this should eventually be addressed"
)]

// Might warrant a standalone crate at some point
//
// This is pub only to make the doctests run (but the crate's pub-ness needs a major overhaul
// anyway)
pub mod oluru;
pub mod seccontext;
