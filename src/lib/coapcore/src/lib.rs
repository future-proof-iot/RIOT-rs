//! A CoAP security for embedded devices, supporting OSCORE/EDHOC and managing credentials.
//!
//! The crate is under heavy development: Its API is in flux. So far, it has hidden dependencies on a
//! particular implementation of the [`coap-message`] provided (it needs to be a
//! [`coap_message_implementations::inmemory_write::Message`]).
#![no_std]

// Might warrant a standalone crate at some point
//
// This is pub only to make the doctests run (but the crate's pub-ness needs a major overhaul
// anyway)
pub mod oluru;
pub mod seccontext;
