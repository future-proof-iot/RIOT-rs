// #![no_std]
#![no_main]

// FAIL: the `autostart` parameter is mandatory
#[riot_rs::thread]
fn main() {}
