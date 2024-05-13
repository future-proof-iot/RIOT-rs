pub(crate) use embassy_executor::InterruptExecutor as Executor;
pub use embassy_stm32::{interrupt, peripherals, Config, OptionalPeripherals, Peripherals};

use embassy_stm32::interrupt::{InterruptExt, Priority};

#[cfg(feature = "usb")]
pub mod usb;

include!(concat!(env!("OUT_DIR"), "/swi.rs"));

use riot_rs_debug::println;
pub fn init(_config: Config) -> OptionalPeripherals {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi48 = Some(Hsi48Config {
            sync_from_usb: true,
        }); // needed for USB
        config.rcc.sys = Sysclk::PLL1_R;
        config.rcc.hse = Some(Hse {
            freq: embassy_stm32::time::Hertz(32000000),
            mode: HseMode::Oscillator,
            prescaler: HsePrescaler::DIV1,
        });
        config.rcc.pll = Some(Pll {
            source: PllSource::HSE,
            prediv: PllPreDiv::DIV2,
            mul: PllMul::MUL10,
            divp: None,
            divq: None,
            divr: Some(PllRDiv::DIV2), // sysclk 80Mhz (32 / 2 * 10 / 2)
        });
        config.rcc.mux.clk48sel = mux::Clk48sel::HSI48;
        //use embassy_stm32::rcc::*;
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
