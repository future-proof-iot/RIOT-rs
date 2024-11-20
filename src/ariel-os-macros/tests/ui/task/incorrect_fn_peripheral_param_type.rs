#![no_main]
#![feature(impl_trait_in_assoc_type)]
#![feature(used_with_arg)]

// FAIL: the function is expected to take a type having a `take_peripherals()` method as first
// parameter
#[ariel_os::task(autostart, peripherals)]
async fn main(_foo: Bar) {}

struct Bar;
