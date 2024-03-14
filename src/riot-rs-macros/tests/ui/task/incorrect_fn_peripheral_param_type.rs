#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

// FAIL: the function is expected to take a type having a `take_peripherals()` method as first
// parameter
#[riot_rs::task(autostart, peripherals)]
async fn main(_foo: Bar) {}

struct Bar;
