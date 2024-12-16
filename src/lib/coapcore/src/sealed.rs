//! Helpers used for sealing in this crate
//!
//! Terminology is taken from the self-proclaimed ["definitive
//! guide"](https://predr.ag/blog/definitive-guide-to-sealed-traits-in-rust/).

/// The trait used in the "supertrait sealed trait" pattern for this crate.
pub trait Sealed {}

/// The type used in the "method signature sealed trait" pattern for this crate.
#[derive(Debug, Copy, Clone)]
pub struct PrivateMethod;
