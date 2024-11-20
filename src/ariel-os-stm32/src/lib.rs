//! Items specific to the STMicroelectronics STM32 MCUs.

#![no_std]
#![feature(doc_auto_cfg)]
#![feature(type_alias_impl_trait)]
#![deny(missing_docs)]

pub mod gpio;

#[doc(hidden)]
pub mod peripheral {
    pub use embassy_stm32::Peripheral;
}

#[cfg(feature = "external-interrupts")]
#[doc(hidden)]
pub mod extint_registry;

#[cfg(feature = "i2c")]
pub mod i2c;

#[doc(hidden)]
pub mod identity;

#[cfg(feature = "spi")]
pub mod spi;

use embassy_stm32::Config;

#[doc(hidden)]
pub use embassy_stm32::{interrupt, OptionalPeripherals, Peripherals};

pub use embassy_stm32::peripherals;

#[cfg(feature = "executor-interrupt")]
pub(crate) use embassy_executor::InterruptExecutor as Executor;

#[cfg(feature = "hwrng")]
#[doc(hidden)]
pub mod hwrng;

#[cfg(feature = "usb")]
cfg_if::cfg_if! {
    if #[cfg(feature = "stm32-usb")] {
        #[doc(hidden)]
        #[path = "usb.rs"]
        pub mod usb;
    } else if #[cfg(feature = "stm32-usb-synopsis")] {
        #[doc(hidden)]
        #[path = "usb_synopsis_otg.rs"]
        pub mod usb;
    } else {
        compile_error!("stm32: usb enabled but no flavor selected. Choose `stm32-usb` or `stm32-usb-synopsis`.");
    }
}

#[cfg(feature = "executor-interrupt")]
include!(concat!(env!("OUT_DIR"), "/swi.rs"));

#[cfg(capability = "hw/stm32-dual-core")]
use {core::mem::MaybeUninit, embassy_stm32::SharedData};

// Ariel OS doesn't support the second core yet, but upstream needs this.
#[cfg(capability = "hw/stm32-dual-core")]
static SHARED_DATA: MaybeUninit<SharedData> = MaybeUninit::uninit();

#[cfg(feature = "executor-interrupt")]
pub static EXECUTOR: Executor = Executor::new();

#[doc(hidden)]
pub fn init() -> OptionalPeripherals {
    let mut config = Config::default();
    board_config(&mut config);

    #[cfg(not(capability = "hw/stm32-dual-core"))]
    let peripherals = embassy_stm32::init(config);

    #[cfg(capability = "hw/stm32-dual-core")]
    let peripherals = embassy_stm32::init_primary(config, &SHARED_DATA);

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
            // Required for SPI (configured by `spi123sel`)
            divq: Some(PllDiv::DIV16), // FIXME: adjust this divider
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
        // Select the clock signal used for SPI1, SPI2, and SPI3.
        // FIXME: what to do about SPI4, SPI5, and SPI6?
        config.rcc.mux.spi123sel = mux::Saisel::PLL1_Q; // Reset value
    }

    // mark used
    let _ = config;
}
