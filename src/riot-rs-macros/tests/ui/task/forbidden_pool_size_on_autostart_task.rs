#![no_main]
#![feature(impl_trait_in_assoc_type)]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

// FAIL: the `pool_size` parameter cannot be used on autostart task
#[riot_rs::task(autostart, pool_size = 4)]
async fn main() {}
