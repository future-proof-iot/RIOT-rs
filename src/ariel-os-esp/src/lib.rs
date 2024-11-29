//! Items specific to the Espressif ESP MCUs.

#![no_std]
#![feature(doc_auto_cfg)]
#![feature(impl_trait_in_assoc_type)]
#![feature(type_alias_impl_trait)]
#![deny(missing_docs)]

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

pub mod peripherals {
    //! Types for the peripheral singletons.

    #![expect(missing_docs)]

    pub use esp_hal::peripherals::*;

    pub type GPIO0 = esp_hal::gpio::GpioPin<0>;
    pub type GPIO1 = esp_hal::gpio::GpioPin<1>;
    pub type GPIO2 = esp_hal::gpio::GpioPin<2>;
    pub type GPIO3 = esp_hal::gpio::GpioPin<3>;
    pub type GPIO4 = esp_hal::gpio::GpioPin<4>;
    pub type GPIO5 = esp_hal::gpio::GpioPin<5>;
    pub type GPIO6 = esp_hal::gpio::GpioPin<6>;
    pub type GPIO7 = esp_hal::gpio::GpioPin<7>;
    pub type GPIO8 = esp_hal::gpio::GpioPin<8>;
    pub type GPIO9 = esp_hal::gpio::GpioPin<9>;

    pub type GPIO10 = esp_hal::gpio::GpioPin<10>;
    pub type GPIO11 = esp_hal::gpio::GpioPin<11>;
    pub type GPIO12 = esp_hal::gpio::GpioPin<12>;
    pub type GPIO13 = esp_hal::gpio::GpioPin<13>;
    pub type GPIO14 = esp_hal::gpio::GpioPin<14>;
    pub type GPIO15 = esp_hal::gpio::GpioPin<15>;
    pub type GPIO16 = esp_hal::gpio::GpioPin<16>;
    pub type GPIO17 = esp_hal::gpio::GpioPin<17>;
    pub type GPIO18 = esp_hal::gpio::GpioPin<18>;
    pub type GPIO19 = esp_hal::gpio::GpioPin<19>;
    pub type GPIO20 = esp_hal::gpio::GpioPin<20>;
    pub type GPIO21 = esp_hal::gpio::GpioPin<21>;

    cfg_if::cfg_if! {
        if #[cfg(context = "esp32c6")] {
            pub type GPIO22 = esp_hal::gpio::GpioPin<22>;
            pub type GPIO23 = esp_hal::gpio::GpioPin<23>;
            pub type GPIO24 = esp_hal::gpio::GpioPin<24>;
            pub type GPIO25 = esp_hal::gpio::GpioPin<25>;
            pub type GPIO26 = esp_hal::gpio::GpioPin<26>;
            pub type GPIO27 = esp_hal::gpio::GpioPin<27>;
            pub type GPIO28 = esp_hal::gpio::GpioPin<28>;
            pub type GPIO29 = esp_hal::gpio::GpioPin<29>;
            pub type GPIO30 = esp_hal::gpio::GpioPin<30>;
        }
        else if #[cfg(context = "esp32")] {
            pub type GPIO22 = esp_hal::gpio::GpioPin<22>;
            pub type GPIO23 = esp_hal::gpio::GpioPin<23>;
            pub type GPIO24 = esp_hal::gpio::GpioPin<24>;
            pub type GPIO25 = esp_hal::gpio::GpioPin<25>;
            pub type GPIO26 = esp_hal::gpio::GpioPin<26>;
            pub type GPIO27 = esp_hal::gpio::GpioPin<27>;
            pub type GPIO32 = esp_hal::gpio::GpioPin<32>;
            pub type GPIO33 = esp_hal::gpio::GpioPin<33>;
            pub type GPIO34 = esp_hal::gpio::GpioPin<34>;
            pub type GPIO35 = esp_hal::gpio::GpioPin<35>;
            pub type GPIO36 = esp_hal::gpio::GpioPin<36>;
            pub type GPIO37 = esp_hal::gpio::GpioPin<37>;
            pub type GPIO38 = esp_hal::gpio::GpioPin<38>;
            pub type GPIO39 = esp_hal::gpio::GpioPin<39>;
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
