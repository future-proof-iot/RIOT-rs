#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use riot_rs::debug::log::*;

#[riot_rs::thread(autostart)]
fn main() {
    use rand::Rng as _;
    let mut rng = riot_rs::random::fast_rng();

    let value = rng.gen_range(1..=6);

    info!("The random value of this round is {}.", value);
}
