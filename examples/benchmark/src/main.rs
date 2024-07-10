#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use riot_rs::debug::log::*;

#[riot_rs::thread(autostart)]
fn main() {
    match riot_rs::bench::benchmark(1000, || {
        // Insert the function to benchmark here.
        // Consider using `core::hint::black_box()` where necessary.
    }) {
        Ok(ticks) => {
            info!("took {} per iteration", ticks);
        }
        Err(err) => {
            info!("benchmark returned error: {}", err);
        }
    }
}
