pub mod gpio;
pub mod i2c;

pub mod peripheral {
    pub use embassy_stm32::Peripheral;
}

use embassy_stm32::Config;

pub use embassy_stm32::{interrupt, peripherals, OptionalPeripherals, Peripherals};

#[cfg(feature = "executor-interrupt")]
pub(crate) use embassy_executor::InterruptExecutor as Executor;

#[cfg(feature = "usb")]
cfg_if::cfg_if! {
    if #[cfg(feature = "stm32-usb")] {
        #[path = "usb.rs"]
        pub mod usb;
    } else if #[cfg(feature = "stm32-usb-synopsis")] {
        #[path = "usb_synopsis_otg.rs"]
        pub mod usb;
    } else {
        compile_error!("stm32: usb enabled but no flavor selected. Choose `stm32-usb` or `stm32-usb-synopsis`.");
    }
}

#[cfg(feature = "executor-interrupt")]
include!(concat!(env!("OUT_DIR"), "/swi.rs"));

pub fn init() -> OptionalPeripherals {
    let mut config = Config::default();
    board_config(&mut config);
    let peripherals = embassy_stm32::init(config);
    OptionalPeripherals::from(peripherals)
}

// TODO: find better place for this
fn board_config(config: &mut Config) {
    #[cfg(builder = "st-nucleo-wb55")]
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
    }

    #[cfg(context = "stm32h755zitx")]
    {
        use embassy_stm32::rcc::*;
        use riot_rs_debug::println;

        config.rcc.hsi = Some(HSIPrescaler::DIV1);
        config.rcc.csi = true;
        config.rcc.hsi48 = Some(Hsi48Config {
            sync_from_usb: true,
        }); // needed for USB
        config.rcc.pll1 = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL50,
            divp: Some(PllDiv::DIV2),
            divq: None,
            divr: None,
        });
        config.rcc.sys = Sysclk::PLL1_P; // 400 Mhz
        config.rcc.ahb_pre = AHBPrescaler::DIV2; // 200 Mhz
        config.rcc.apb1_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb2_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb3_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb4_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.voltage_scale = VoltageScale::Scale1;
        // Set SMPS power config otherwise MCU will not powered after next power-off
        config.rcc.supply_config = SupplyConfig::DirectSMPS;
        config.rcc.mux.usbsel = mux::Usbsel::HSI48;
    }

    // mark used
    let _ = config;
}
