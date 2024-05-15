pub mod gpio;

use esp_hal::{clock::ClockControl, embassy, prelude::*, timer::TimerGroup};

pub use esp_hal::{
    embassy::executor::Executor,
    peripherals::{OptionalPeripherals, Peripherals},
};

pub fn init() -> OptionalPeripherals {
    let mut peripherals = OptionalPeripherals::from(Peripherals::take());
    let system = peripherals.SYSTEM.take().unwrap().split();
    let clocks = ClockControl::max(system.clock_control).freeze();

    #[cfg(feature = "wifi-esp")]
    {
        use esp_hal::Rng;
        use esp_wifi::{initialize, EspWifiInitFor};

        riot_rs_debug::println!("riot-rs-embassy::arch::esp::init(): wifi");

        let timer = esp_hal::systimer::SystemTimer::new(peripherals.SYSTIMER.take().unwrap());

        #[cfg(target_arch = "riscv32")]
        let init = initialize(
            EspWifiInitFor::Wifi,
            timer.alarm0,
            Rng::new(peripherals.RNG.take().unwrap()),
            system.radio_clock_control,
            &clocks,
        )
        .unwrap();

        crate::wifi::esp_wifi::WIFI_INIT.set(init).unwrap();
    }

    let timer_group0 = TimerGroup::new(peripherals.TIMG0.take().unwrap(), &clocks);
    embassy::init(&clocks, timer_group0);

    peripherals
}
