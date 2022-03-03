#![no_std]
#![no_main]

use riot_rs as _;

use embedded_hal::blocking::delay::DelayMs;

use riot_wrappers::{stdio::println, ztimer};

#[no_mangle]
fn riot_main() {
    let mut delay = ztimer::Clock::msec();

    let mut bottles = 99;

    while bottles > 0 {
        delay.delay_ms(1000);
        println!(
            "{} bottles of beer on the wall, {} bottles of beer",
            bottles, bottles
        );
        delay.delay_ms(200);
        bottles -= 1;
        println!("Take one down, pass it around: {} bottles of beer", bottles);
    }
}
