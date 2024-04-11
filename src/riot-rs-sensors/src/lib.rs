#![no_std]
// Required by linkme
#![feature(used_with_arg)]
#![feature(error_in_core)]
// Used by `Registry::read_all()`
#![feature(impl_trait_in_assoc_type)]
// Cast to &dyn Any from sensor refs
#![feature(trait_upcasting)]
#![deny(unused_must_use)]

pub mod registry;
pub mod sensor;
