#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use riot_rs::debug::println;

#[riot_rs::thread]
fn main() {
    match riot_rs::bench::benchmark(1000, || {
        let mut i = 0;

        // This seems to be measuring branching cost more than anything
        for _ in 0..1000 {
            i += 1;
            core::hint::black_box(i);
        }
    }) {
        Ok(ticks) => {
            println!("took {} per iteration", ticks);
        }
        Err(err) => {
            println!("benchmark returned error: {}", err);
        }
    }
}
