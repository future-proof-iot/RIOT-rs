#![no_main]
#![no_std]
#![feature(used_with_arg)]

use ariel_os::debug::log::*;

#[ariel_os::thread(autostart)]
fn main() {
    match ariel_os::bench::benchmark(1000, || {
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
