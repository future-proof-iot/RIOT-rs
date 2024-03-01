use esp_hal::{
    clock::ClockControl,
    embassy::{
        self,
        executor::{FromCpu1, InterruptExecutor},
    },
    prelude::*,
    timer::TimerGroup,
};

pub(crate) use esp_hal::interrupt::{self};
pub use esp_hal::peripherals::{OptionalPeripherals, Peripherals};

pub(crate) type Executor = InterruptExecutor<FromCpu1>;
pub static SWI: () = ();

#[derive(Default)]
pub struct Config {}

#[interrupt]
fn FROM_CPU_INTR1() {
    unsafe { crate::EXECUTOR.on_interrupt() }
}

pub fn init(_config: Config) -> OptionalPeripherals {
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
