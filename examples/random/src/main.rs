#![no_main]
#![no_std]
#![feature(used_with_arg)]

use ariel_os::debug::log::*;

#[ariel_os::thread(autostart)]
fn main() {
    use rand::Rng as _;
    let mut rng = ariel_os::random::fast_rng();

    let value = rng.gen_range(1..=6);

    info!("The random value of this round is {}.", value);
}
