#![no_std]
#![feature(doc_auto_cfg)]
#![feature(impl_trait_in_assoc_type)]
#![feature(type_alias_impl_trait)]

pub mod gpio;

#[cfg(feature = "hwrng")]
#[doc(hidden)]
pub mod hwrng {
    pub fn construct_rng(_peripherals: &mut crate::OptionalPeripherals) {
        // handled in `init()`
    }
}

#[cfg(feature = "i2c")]
pub mod i2c;

#[doc(hidden)]
pub mod identity {
    use ariel_os_embassy_common::identity;

    pub type DeviceId = identity::NoDeviceId<identity::NotImplemented>;
}

#[cfg(feature = "spi")]
pub mod spi;

#[cfg(feature = "usb")]
#[doc(hidden)]
pub mod usb;

#[cfg(feature = "wifi")]
#[doc(hidden)]
pub mod wifi;

#[doc(hidden)]
pub mod peripheral {
    pub use esp_hal::peripheral::Peripheral;
}

#[doc(hidden)]
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
        else if #[cfg(context = "esp32")] {
            pub use esp_hal::gpio::GPIO_22;
            pub use esp_hal::gpio::GPIO_23;
            pub use esp_hal::gpio::GPIO_24;
            pub use esp_hal::gpio::GPIO_25;
            pub use esp_hal::gpio::GPIO_26;
            pub use esp_hal::gpio::GPIO_27;
            pub use esp_hal::gpio::GPIO_32;
            pub use esp_hal::gpio::GPIO_33;
            pub use esp_hal::gpio::GPIO_34;
            pub use esp_hal::gpio::GPIO_35;
            pub use esp_hal::gpio::GPIO_36;
            pub use esp_hal::gpio::GPIO_37;
            pub use esp_hal::gpio::GPIO_38;
            pub use esp_hal::gpio::GPIO_39;
        }
    }
}

#[doc(hidden)]
pub use esp_hal::peripherals::OptionalPeripherals;

#[cfg(feature = "executor-single-thread")]
#[doc(hidden)]
pub use esp_hal_embassy::Executor;

#[doc(hidden)]
pub fn init() -> OptionalPeripherals {
    let mut config = esp_hal::Config::default();
    config.cpu_clock = esp_hal::clock::CpuClock::max();

    let mut peripherals = OptionalPeripherals::from(esp_hal::init(config));

    #[cfg(any(feature = "hwrng", feature = "wifi-esp"))]
    let rng = esp_hal::rng::Rng::new(peripherals.RNG.take().unwrap());

    #[cfg(feature = "hwrng")]
    ariel_os_random::construct_rng(rng);

    #[cfg(feature = "wifi-esp")]
    {
        use esp_hal::timer::timg::TimerGroup;

        use esp_alloc as _;
        esp_alloc::heap_allocator!(72 * 1024);

        use esp_wifi::{init, EspWifiInitFor};

        ariel_os_debug::log::debug!("ariel-os-embassy::hal::esp::init(): wifi");

        let timer = TimerGroup::new(peripherals.TIMG0.take().unwrap()).timer0;

        let init = init(
            EspWifiInitFor::Wifi,
            timer,
            rng,
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
