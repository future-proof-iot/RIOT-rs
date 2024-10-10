#![no_std]
#![feature(doc_auto_cfg)]
#![feature(impl_trait_in_assoc_type)]
#![feature(trait_alias)]
#![feature(type_alias_impl_trait)]

pub mod gpio;

#[cfg(feature = "i2c")]
pub mod i2c;

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

pub use esp_hal::peripherals::OptionalPeripherals;

#[cfg(feature = "executor-single-thread")]
pub use esp_hal_embassy::Executor;

pub fn init() -> OptionalPeripherals {
    let mut peripherals = OptionalPeripherals::from(esp_hal::init(esp_hal::Config::default()));

    #[cfg(feature = "wifi-esp")]
    {
        use esp_hal::timer::timg::TimerGroup;

        use esp_alloc as _;
        esp_alloc::heap_allocator!(72 * 1024);

        use esp_hal::rng::Rng;
        use esp_wifi::{init, EspWifiInitFor};

        riot_rs_debug::log::debug!("riot-rs-embassy::arch::esp::init(): wifi");

        let timer = TimerGroup::new(peripherals.TIMG0.take().unwrap()).timer0;

        let init = init(
            EspWifiInitFor::Wifi,
            timer,
            Rng::new(peripherals.RNG.take().unwrap()),
            peripherals.RADIO_CLK.take().unwrap(),
        )
        .unwrap();

        wifi::esp_wifi::WIFI_INIT.set(init).unwrap();
    }

    let embassy_timer = {
        cfg_if::cfg_if! {
            if #[cfg(context = "esp32")] {
                use esp_hal::timer::timg::TimerGroup;
                TimerGroup::new(peripherals.TIMG1.take().unwrap()).timer0
            } else {
                use esp_hal::timer::systimer::{SystemTimer, Target};
                SystemTimer::new(peripherals.SYSTIMER.take().unwrap()).split::<Target>().alarm0
            }
        }
    };

    esp_hal_embassy::init(embassy_timer);

    peripherals
}
