use embassy_time::{Duration, Timer};
use esp_wifi::{
    wifi::{
        ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent, WifiStaDevice,
        WifiState,
    },
    EspWifiInitialization,
};
use once_cell::sync::OnceCell;
use riot_rs_debug::log::{debug, info};

use crate::{arch::OptionalPeripherals, Spawner};

pub type NetworkDevice = WifiDevice<'static, WifiStaDevice>;

// Ideally, all Wi-Fi initialization would happen here.
// Unfortunately that's complicated, so we're using WIFI_INIT to pass the
// `EspWifiInitialization` from `crate::arch::esp::init()`.
// Using a `once_cell::OnceCell` here for critical-section support, just to be
// sure.
pub static WIFI_INIT: OnceCell<EspWifiInitialization> = OnceCell::new();

#[cfg(feature = "threading")]
pub static WIFI_THREAD_ID: OnceCell<riot_rs_threads::ThreadId> = OnceCell::new();

pub fn init(peripherals: &mut OptionalPeripherals, spawner: Spawner) -> NetworkDevice {
    let wifi = peripherals.WIFI.take().unwrap();
    let init = WIFI_INIT.get().unwrap();
    let (device, controller) = esp_wifi::wifi::new_with_mode(init, wifi, WifiStaDevice).unwrap();

    spawner.spawn(connection(controller)).ok();

    device
}

#[embassy_executor::task]
async fn connection(mut controller: WifiController<'static>) {
    #[cfg(feature = "threading")]
    {
        let thread_id = WIFI_THREAD_ID.get().unwrap();
        riot_rs_threads::wakeup(*thread_id);
    }

    debug!("start connection task");
    debug!("Device capabilities: {:?}", controller.get_capabilities());
    loop {
        match esp_wifi::wifi::get_wifi_state() {
            WifiState::StaConnected => {
                // wait until we're no longer connected
                controller.wait_for_event(WifiEvent::StaDisconnected).await;
                Timer::after(Duration::from_secs(5)).await
            }
            _ => {}
        }
        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = Configuration::Client(ClientConfiguration {
                ssid: crate::wifi::WIFI_NETWORK.try_into().unwrap(),
                password: crate::wifi::WIFI_PASSWORD.try_into().unwrap(),
                ..Default::default()
            });
            controller.set_configuration(&client_config).unwrap();
            debug!("Starting Wi-Fi");
            controller.start().await.unwrap();
            debug!("Wi-Fi started!");
        }
        debug!("About to connect...");

        match controller.connect().await {
            Ok(_) => info!("Wifi connected!"),
            Err(e) => {
                info!("Failed to connect to Wi-Fi: {:?}", e);
                Timer::after(Duration::from_millis(5000)).await
            }
        }
    }
}

#[cfg(feature = "threading")]
mod wifi_thread {

    use super::*;
    #[cfg(context = "esp32c6")]
    use esp_hal::peripherals::INTPRI as SystemPeripheral;
    #[cfg(context = "esp32c3")]
    use esp_hal::peripherals::SYSTEM as SystemPeripheral;
    use esp_hal::{
        interrupt,
        peripherals::{Interrupt, SYSTIMER},
    };

    // Handle the systimer alarm 0 interrupt, configured in esp-wifi.
    extern "C" fn systimer_target0_() {
        unsafe {
            SYSTIMER::steal()
                .int_clr()
                .write(|w| w.target0().clear_bit_by_one())
        }
        // Wakeup `esp_wifi_thread`.
        riot_rs_threads::wakeup(*WIFI_THREAD_ID.get().unwrap());
    }

    // CPU Interrupt 3 triggers the scheduler in `esp-wifi`.
    fn yield_to_esp_wifi_scheduler() {
        unsafe {
            (&*SystemPeripheral::PTR)
                .cpu_intr_from_cpu_3()
                .modify(|_, w| w.cpu_intr_from_cpu_3().set_bit());
        }
    }

    /// High priority thread that frequently wakes up to run the esp-wifi
    /// scheduler.
    fn esp_wifi_thread() {
        // Wait until `embassy` was intialized.
        riot_rs_threads::sleep();

        // Bind the periodic systimer that is configured in esp-wifi to our own handler.
        unsafe {
            interrupt::bind_interrupt(
                Interrupt::SYSTIMER_TARGET0,
                core::mem::transmute(systimer_target0_ as *const ()),
            );
        }

        loop {
            // Yield to the esp-wifi scheduler tasks, so that they get a chance to run.
            yield_to_esp_wifi_scheduler();
            // Sleep until the systimer alarm 0 interrupts again.
            riot_rs_threads::sleep()
        }
    }

    #[linkme::distributed_slice(riot_rs_threads::THREAD_FNS)]
    fn start_esp_wifi_thread() {
        let stack = static_cell::make_static!([0u8; 2048]);
        let id = riot_rs_threads::thread_create_noarg(
            esp_wifi_thread,
            stack,
            riot_rs_threads::SCHED_PRIO_LEVELS as u8 - 1,
        );
        WIFI_THREAD_ID.set(id).unwrap();
    }
}
