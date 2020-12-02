#![no_std]

use core::fmt::Write;
use cortex_m;
use cortex_m_rt;
use hal::pac::Peripherals;

use linkme::distributed_slice;

use nrf52;

use riot_rs_rt::debug::println;
use riot_rs_rt::init::INIT_FUNCS;

use {
    hal::gpio::Level,
    hal::pac::{interrupt, Interrupt},
    hal::uarte::*,
    nrf52840_hal as hal,
};

//use riot_core::console::Serial;

// #[interrupt]
// fn UARTE0_UART0() {
//     println!("UARTE0_UART0");
// }

#[distributed_slice(INIT_FUNCS, 50)]
pub fn nrf52840dk_init() {
    println!("nrf52840dk::init()");
    nrf52::init();

    // let p = Peripherals::take().unwrap();
    // let p0 = hal::gpio::p0::Parts::new(p.P0);

    // p.UARTE0.intenset.write(|w| unsafe { w.bits(1 << 4) });
    // unsafe { hal::pac::NVIC::unmask(Interrupt::UARTE0_UART0) };
    // let mut uarte = Uarte::new(
    //     p.UARTE0,
    //     Pins {
    //         txd: p0.p0_06.into_push_pull_output(Level::High).degrade(),
    //         rxd: p0.p0_08.into_floating_input().degrade(),
    //         cts: None,
    //         rts: None,
    //     },
    //     Parity::EXCLUDED,
    //     Baudrate::BAUD115200,
    // );

    // write!(&mut uarte, "whatever works\n");
}

pub fn linkme_please() {}
