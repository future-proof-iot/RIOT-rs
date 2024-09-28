#![no_std]
#![feature(doc_auto_cfg)]
#![feature(impl_trait_in_assoc_type)]
#![feature(trait_alias)]
#![feature(type_alias_impl_trait)]

use once_cell::sync::OnceCell;

pub mod gpio;

#[cfg(feature = "i2c")]
pub mod i2c;

pub mod identity {
    use riot_rs_embassy_common::identity::*;

    pub type DeviceId = NoDeviceId<NotImplemented>;
}

#[cfg(feature = "wifi")]
pub mod wifi;

pub mod peripheral {
    pub use esp_hal::peripheral::Peripheral;
}

pub mod peripherals {
    pub use esp_hal::peripherals::*;

    pub use esp_hal::gpio::GPIO_0;
    pub use esp_hal::gpio::GPIO_1;
    pub use esp_hal::gpio::GPIO_2;
    pub use esp_hal::gpio::GPIO_3;
    pub use esp_hal::gpio::GPIO_4;
    pub use esp_hal::gpio::GPIO_5;
    pub use esp_hal::gpio::GPIO_6;
    pub use esp_hal::gpio::GPIO_7;
    pub use esp_hal::gpio::GPIO_8;
    pub use esp_hal::gpio::GPIO_9;

    pub use esp_hal::gpio::GPIO_10;
    pub use esp_hal::gpio::GPIO_11;
    pub use esp_hal::gpio::GPIO_12;
    pub use esp_hal::gpio::GPIO_13;
    pub use esp_hal::gpio::GPIO_14;
    pub use esp_hal::gpio::GPIO_15;
    pub use esp_hal::gpio::GPIO_16;
    pub use esp_hal::gpio::GPIO_17;
    pub use esp_hal::gpio::GPIO_18;
    pub use esp_hal::gpio::GPIO_19;
    pub use esp_hal::gpio::GPIO_20;
    pub use esp_hal::gpio::GPIO_21;

    cfg_if::cfg_if! {
        if #[cfg(context = "esp32c6")] {
            pub use esp_hal::gpio::GPIO_22;
            pub use esp_hal::gpio::GPIO_23;
            pub use esp_hal::gpio::GPIO_24;
            pub use esp_hal::gpio::GPIO_25;
            pub use esp_hal::gpio::GPIO_26;
            pub use esp_hal::gpio::GPIO_27;
            pub use esp_hal::gpio::GPIO_28;
            pub use esp_hal::gpio::GPIO_29;
            pub use esp_hal::gpio::GPIO_30;
        }
    }
}

use esp_hal::{
    clock::{ClockControl, Clocks},
    system::SystemControl,
    timer::timg::TimerGroup,
};

pub use esp_hal::peripherals::OptionalPeripherals;

#[cfg(feature = "executor-single-thread")]
pub use esp_hal_embassy::Executor;

// NOTE(once-cell): using a `once_cell::OnceCell` here for critical-section support, just to be
// sure.
pub(crate) static CLOCKS: OnceCell<Clocks> = OnceCell::new();

pub fn init() -> OptionalPeripherals {
    let mut peripherals = OptionalPeripherals::from(peripherals::Peripherals::take());
    let system = SystemControl::new(peripherals.SYSTEM.take().unwrap());
    let clocks = ClockControl::max(system.clock_control).freeze();

    #[cfg(feature = "wifi-esp")]
    {
        use esp_hal::{rng::Rng, timer::systimer::SystemTimer};
        use esp_wifi::{initialize, EspWifiInitFor};

        riot_rs_debug::log::debug!("riot-rs-embassy::arch::esp::init(): wifi");

        let timer = SystemTimer::new(peripherals.SYSTIMER.take().unwrap())
            .split::<esp_hal::timer::systimer::Target>();

        #[cfg(target_arch = "riscv32")]
        let init = initialize(
            EspWifiInitFor::Wifi,
            timer.alarm0,
            Rng::new(peripherals.RNG.take().unwrap()),
            peripherals.RADIO_CLK.take().unwrap(),
            &clocks,
        )
        .unwrap();

        wifi::esp_wifi::WIFI_INIT.set(init).unwrap();
    }

    let timer_group0 = TimerGroup::new(peripherals.TIMG0.take().unwrap(), &clocks);
    esp_hal_embassy::init(&clocks, timer_group0.timer0);

    // Discard the error in (the impossible) case that it was already populated.
    let _ = CLOCKS.set(clocks);

    peripherals
}
