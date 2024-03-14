#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

// FAIL: the function must be async
#[riot_rs::task]
fn main() {}
