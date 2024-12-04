//! A CoAP security for embedded devices, supporting OSCORE/EDHOC and managing credentials.
//!
//! The crate is under heavy development: Its API is in flux. So far, it has hidden dependencies on a
//! particular implementation of the [`coap-message`] provided (it needs to be a
//! [`coap_message_implementations::inmemory_write::Message`]).
//!
//! # Logging
//!
//! Extensive logging is available in this crate through [`defmt_or_log`], depending on features
//! enabled.
//!
//! Errors from CoAP are currently logged through its [`Debug2Format`](defmt_or_log::Debug2Format)
//! facility, representing a compromise between development and runtime complexity. Should
//! benchmarks show this to be a significant factor in code size in applications that need error
//! handling, more fine grained control can be implemented (eg. offering an option to make
//! Debug2Format merely print the type name or even make it empty).
//!
//! See the book for [how defmt is configured in
//! Ariel OS](https://ariel-os.github.io/ariel-os/dev/docs/book/tooling/defmt.html).
//!
//! **Warning**: At the Debug level, this module may show cryptographic key material. This will be
//! revised once all components have been interop-tested.
#![doc = document_features::document_features!(feature_label = r#"<span class="stab portability"><code>{feature}</code></span>"#)]
#![no_std]
#![cfg_attr(feature = "_nightly_docs", feature(doc_auto_cfg))]

// Might warrant a standalone crate at some point
//
// This is pub only to make the doctests run (but the crate's pub-ness needs a major overhaul
// anyway)
pub mod oluru;
pub mod seccontext;
