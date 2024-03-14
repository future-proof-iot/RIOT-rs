#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

// FAIL: the `autostart` parameter must be present when requesting peripherals
#[riot_rs::task(peripherals)]
async fn main(_foo: Bar) {}

struct Bar;
