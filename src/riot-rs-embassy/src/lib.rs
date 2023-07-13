#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use embassy_executor::InterruptExecutor;

pub static EXECUTOR: InterruptExecutor = InterruptExecutor::new();

pub mod blocker;

#[cfg(context = "nrf52")]
use embassy_nrf as embassy_arch;
#[cfg(context = "nrf52")]
use embassy_nrf::interrupt::SWI0_EGU0 as SWI;

#[cfg(context = "rp2040")]
use embassy_rp as embassy_arch;
#[cfg(context = "rp2040")]
use embassy_rp::interrupt::SWI_IRQ_1 as SWI;

use embassy_arch::interrupt;

#[cfg(context = "nrf52")]
#[interrupt]
unsafe fn SWI0_EGU0() {
    EXECUTOR.on_interrupt()
}

#[cfg(context = "rp2040")]
#[interrupt]
unsafe fn SWI_IRQ_1() {
    EXECUTOR.on_interrupt()
}

// #[cfg(context = "rp2040")]
// #[embassy_executor::task]
// async fn embassy_init(p: Peripherals) {
//     use embassy_rp::uart::{Config, UartTx};
//     use embassy_time::{Duration, Timer};
//     let mut uart_tx = UartTx::new(p.UART0, p.PIN_0, p.DMA_CH0, Config::default());

//     loop {
//         let data = b"hello\n";
//         uart_tx.write(&data[..]).await.unwrap();
//         Timer::after(Duration::from_secs(1)).await;
//     }
// }

pub(crate) fn init() {
    riot_rs_rt::debug::println!("riot-rs-embassy::init()");
    let _p = embassy_arch::init(Default::default());

    EXECUTOR.start(SWI);
    // #[cfg(context = "rp2040")]
    // {
    //     EXECUTOR.spawner().spawn(embassy_init(_p)).unwrap();
    // }
}

use linkme::distributed_slice;
use riot_rs_rt::INIT_FUNCS;

#[distributed_slice(INIT_FUNCS)]
static RIOT_RS_EMBASSY_INIT: fn() = init;
