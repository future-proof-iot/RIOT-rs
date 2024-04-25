pub(crate) use embassy_executor::InterruptExecutor as Executor;
pub use embassy_stm32::interrupt;
pub use embassy_stm32::interrupt::LPUART1 as SWI;
pub use embassy_stm32::{peripherals, Config, OptionalPeripherals, Peripherals};

use embassy_stm32::interrupt::{InterruptExt, Priority};

#[interrupt]
unsafe fn LPUART1() {
    // SAFETY:
    // - called from ISR
    // - not called before `start()`, as the interrupt is enabled by `start()`
    //   itself
    unsafe { crate::EXECUTOR.on_interrupt() }
}

use riot_rs_debug::println;
pub fn init(_config: Config) -> OptionalPeripherals {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        // config.rcc.hsi = Some(HSIPrescaler::DIV1);
        // config.rcc.csi = true;
        // config.rcc.hsi48 = Some(Hsi48Config {
        //     sync_from_usb: true,
        // }); // needed for USB
        // config.rcc.pll1 = Some(Pll {
        //     source: PllSource::HSI,
        //     prediv: PllPreDiv::DIV4,
        //     mul: PllMul::MUL50,
        //     divp: Some(PllDiv::DIV2),
        //     divq: None,
        //     divr: None,
        // });
        // config.rcc.sys = Sysclk::PLL1_P; // 400 Mhz
        // config.rcc.ahb_pre = AHBPrescaler::DIV2; // 200 Mhz
        // config.rcc.apb1_pre = APBPrescaler::DIV2; // 100 Mhz
        // config.rcc.apb2_pre = APBPrescaler::DIV2; // 100 Mhz
        // config.rcc.apb3_pre = APBPrescaler::DIV2; // 100 Mhz
        // config.rcc.apb4_pre = APBPrescaler::DIV2; // 100 Mhz
        // config.rcc.voltage_scale = VoltageScale::Scale1;
        // config.rcc.mux.usbsel = mux::Usbsel::HSI48;
    }
    println!("{}{}", file!(), line!());
    let peripherals = embassy_stm32::init(config);
    println!("{}{}", file!(), line!());
    OptionalPeripherals::from(peripherals)
}
