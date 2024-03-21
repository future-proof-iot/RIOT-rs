#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use riot_rs::debug::println;

#[riot_rs::thread(autostart)]
fn main() {
    match riot_rs::bench::benchmark(1000, || {
        // Insert the function to benchmark here.
        // Consider using `core::hint::black_box()` where necessary.
    }) {
        Ok(ticks) => {
            println!("took {} per iteration", ticks);
        }
        Err(err) => {
            println!("benchmark returned error: {}", err);
        }
    }
}
