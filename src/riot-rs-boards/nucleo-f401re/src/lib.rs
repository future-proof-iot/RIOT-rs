#![no_std]

use crate::hal::{prelude::*, serial::config::Config, serial::Serial, stm32};
use stm32f4xx::{hal, peripheral};

use riot_rs_rt::debug::println;

use core::fmt::Write;

pub fn init() {
    println!("boards::nucleo-f401re::init()").unwrap();
    let dp = stm32::Peripherals::take().unwrap();

    let rcc = dp.RCC.constrain();

    rcc.cfgr.use_hse(8.mhz()).freeze();
}
