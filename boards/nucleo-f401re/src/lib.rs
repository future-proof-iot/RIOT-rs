#![no_std]

use crate::hal::{prelude::*, serial::config::Config, serial::Serial, stm32};
use stm32f4xx::{hal, peripheral};

use testing::println;

use core::fmt::Write;

pub fn init() {
    println!("boards::nucleo-f401re::init()").unwrap();
    let dp = stm32::Peripherals::take().unwrap();
    //let cp = peripheral::Peripherals::take().unwrap();

    let rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.use_hse(8.mhz()).freeze();

    //let mut delay = hal::delay::Delay::new(cp.SYST, clocks);

    // define RX/TX pins
    let gpioa = dp.GPIOA.split();
    let tx_pin = gpioa.pa2.into_alternate_af7();
    let rx_pin = gpioa.pa3.into_alternate_af7();

    // configure serial
    let serial = Serial::usart2(
        dp.USART2,
        (tx_pin, rx_pin),
        Config::default().baudrate(115200.bps()),
        clocks,
    )
    .unwrap();

    let (mut tx, mut _rx) = serial.split();

    write!(&mut tx, "serial test").unwrap();
}
