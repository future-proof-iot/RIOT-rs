pub mod gpio;

pub mod peripheral {
    pub use esp_hal::peripheral::Peripheral;
}

use esp_hal::{clock::ClockControl, system::SystemControl, timer::timg::TimerGroup};

pub use esp_hal::peripherals::{OptionalPeripherals, Peripherals};

#[cfg(feature = "executor-single-thread")]
pub use esp_hal_embassy::Executor;

pub fn init() -> OptionalPeripherals {
    let mut peripherals = OptionalPeripherals::from(Peripherals::take());
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
