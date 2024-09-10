#![no_main]
#![feature(impl_trait_in_assoc_type)]
#![feature(used_with_arg)]

// FAIL: the function must be async
#[riot_rs::task]
fn main() {}
