pub mod gpio;

pub mod peripheral {
    pub use esp_hal::peripheral::Peripheral;
}

pub mod peripherals {
    pub use esp_hal::peripherals::*;

    pub use esp_hal::gpio::Gpio0;
    pub use esp_hal::gpio::Gpio1;
    pub use esp_hal::gpio::Gpio2;
    pub use esp_hal::gpio::Gpio3;
    pub use esp_hal::gpio::Gpio4;
    pub use esp_hal::gpio::Gpio5;
    pub use esp_hal::gpio::Gpio6;
    pub use esp_hal::gpio::Gpio7;
    pub use esp_hal::gpio::Gpio8;
    pub use esp_hal::gpio::Gpio9;
    pub use esp_hal::gpio::Gpio10;
    pub use esp_hal::gpio::Gpio11;
    pub use esp_hal::gpio::Gpio12;
    pub use esp_hal::gpio::Gpio13;
    pub use esp_hal::gpio::Gpio14;
    pub use esp_hal::gpio::Gpio15;
    pub use esp_hal::gpio::Gpio16;
    pub use esp_hal::gpio::Gpio17;
    pub use esp_hal::gpio::Gpio18;
    pub use esp_hal::gpio::Gpio19;
    pub use esp_hal::gpio::Gpio20;
    pub use esp_hal::gpio::Gpio21;
    pub use esp_hal::gpio::Gpio22;
    pub use esp_hal::gpio::Gpio23;
    pub use esp_hal::gpio::Gpio24;
    pub use esp_hal::gpio::Gpio25;
    pub use esp_hal::gpio::Gpio26;
    pub use esp_hal::gpio::Gpio27;
    pub use esp_hal::gpio::Gpio28;
    pub use esp_hal::gpio::Gpio29;
    pub use esp_hal::gpio::Gpio30;
}

use esp_hal::{clock::ClockControl, system::SystemControl, timer::timg::TimerGroup};

pub use esp_hal::peripherals::OptionalPeripherals;

#[cfg(feature = "executor-single-thread")]
pub use esp_hal_embassy::Executor;

pub fn init() -> OptionalPeripherals {
    let mut peripherals = OptionalPeripherals::from(peripherals::Peripherals::take());
    let system = SystemControl::new(peripherals.SYSTEM.take().unwrap());
    let clocks = ClockControl::max(system.clock_control).freeze();

    #[cfg(feature = "wifi-esp")]
    {
        use esp_hal::{rng::Rng, timer::systimer::SystemTimer};
        use esp_wifi::{initialize, EspWifiInitFor};

        riot_rs_debug::log::debug!("riot-rs-embassy::arch::esp::init(): wifi");

        let timer = SystemTimer::new(peripherals.SYSTIMER.take().unwrap());

        #[cfg(target_arch = "riscv32")]
        let init = initialize(
            EspWifiInitFor::Wifi,
            timer.alarm0,
            Rng::new(peripherals.RNG.take().unwrap()),
            peripherals.RADIO_CLK.take().unwrap(),
            &clocks,
        )
        .unwrap();

        crate::wifi::esp_wifi::WIFI_INIT.set(init).unwrap();
    }

    let timer_group0 = TimerGroup::new_async(peripherals.TIMG0.take().unwrap(), &clocks);
    esp_hal_embassy::init(&clocks, timer_group0);

    peripherals
}
