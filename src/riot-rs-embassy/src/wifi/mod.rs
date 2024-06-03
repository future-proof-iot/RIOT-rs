#[cfg(feature = "wifi-cyw43")]
pub mod cyw43;
#[cfg(feature = "wifi-esp")]
pub mod esp_wifi;

use riot_rs_utils::str_from_env;

#[cfg(feature = "wifi-cyw43")]
pub(crate) use cyw43::NetworkDevice;

#[cfg(feature = "wifi-esp")]
pub(crate) use esp_wifi::NetworkDevice;

pub(crate) const WIFI_NETWORK: &str =
    str_from_env!("CONFIG_WIFI_NETWORK", "Wi-Fi SSID (network name)");
pub(crate) const WIFI_PASSWORD: &str = str_from_env!("CONFIG_WIFI_PASSWORD", "Wi-Fi password");
