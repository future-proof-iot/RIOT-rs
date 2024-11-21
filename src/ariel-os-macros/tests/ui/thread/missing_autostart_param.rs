// #![no_std]
#![no_main]

// FAIL: the `autostart` parameter is mandatory
#[ariel_os::thread]
fn main() {}
