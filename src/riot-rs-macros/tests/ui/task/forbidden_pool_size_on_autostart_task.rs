#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

// FAIL: the `pool_size` parameter cannot be used on autostart task
#[riot_rs::task(autostart, pool_size = 4)]
async fn main() {}
