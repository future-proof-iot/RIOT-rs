#![no_std]

#[no_mangle]
extern "C" fn pm_set_lowest() {}

pub fn init() {
    riot_rs_rt::debug::println!("lm3s6965ev::init()");
}
