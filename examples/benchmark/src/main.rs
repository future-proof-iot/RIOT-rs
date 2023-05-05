#![no_main]
#![no_std]

use riot_rs::debug::println;

#[no_mangle]
fn riot_main() {
    match riot_rs::rt::benchmark(10000, || {
        //
    }) {
        Ok(ticks) => {
            println!("took {} per iteration", ticks);
        }
        Err(_) => {
            println!("benchmark returned error");
        }
    }
}
