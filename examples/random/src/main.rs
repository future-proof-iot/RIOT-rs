#![no_main]
#![no_std]
#![feature(impl_trait_in_assoc_type)]
#![feature(used_with_arg)]

use ariel_os::{debug::log::*, random::*};

#[ariel_os::task(autostart)]
async fn main() {
    let mut rng = fast_rng();

    for _ in 0..10 {
        let value = rng.gen_range(1..=6);
        info!("The random value of this round is {}.", value);
    }
}
