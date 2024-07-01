use esp_hal::peripherals;
use esp_wifi::{
    wifi::{WifiController, WifiDevice, WifiStaDevice},
    EspWifiInitialization,
};
use once_cell::sync::OnceCell;

use crate::Spawner;

pub type NetworkDevice = WifiDevice<'static, WifiStaDevice>;

// Ideally, all Wi-Fi initialization would happen here.
// Unfortunately that's complicated, so we're using WIFI_INIT to pass the
// `EspWifiInitialization` from `crate::arch::esp::init()`.
// Using a `once_cell::OnceCell` here for critical-section support, just to be
// sure.
pub static WIFI_INIT: OnceCell<EspWifiInitialization> = OnceCell::new();

crate::define_peripherals!(Peripherals { wifi: WIFI });

pub fn init(peripherals: Peripherals, spawner: Spawner) -> NetworkDevice {
    let wifi = peripherals.wifi;
    let init = WIFI_INIT.get().unwrap();
    let (device, controller) = esp_wifi::wifi::new_with_mode(init, wifi, WifiStaDevice).unwrap();

    spawner.spawn(connection(controller)).ok();

    device
}

#[embassy_executor::task]
async fn connection(mut controller: WifiController<'static>) {
    use riot_rs_debug::println;

    use embassy_time::{Duration, Timer};
    use esp_wifi::wifi::{ClientConfiguration, Configuration, WifiEvent, WifiState};

    println!("start connection task");
    println!("Device capabilities: {:?}", controller.get_capabilities());
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
            println!("Starting Wi-Fi");
            controller.start().await.unwrap();
            println!("Wi-Fi started!");
        }
        println!("About to connect...");

        match controller.connect().await {
            Ok(_) => println!("Wifi connected!"),
            Err(e) => {
                println!("Failed to connect to Wi-Fi: {e:?}");
                Timer::after(Duration::from_millis(5000)).await
            }
        }
    }
}
