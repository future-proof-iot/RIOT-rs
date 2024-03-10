#[cfg(feature = "wifi-cyw43")]
pub mod cyw43;
#[cfg(feature = "wifi-esp")]
pub mod esp_wifi;

#[cfg(feature = "wifi-cyw43")]
pub(crate) use cyw43::NetworkDevice;

#[cfg(feature = "wifi-esp")]
pub(crate) use esp_wifi::NetworkDevice;

use riot_rs_utils::str_from_env_or;

pub(crate) const WIFI_NETWORK: &str = str_from_env_or!(
    "CONFIG_WIFI_NETWORK",
    "test_network",
    "Wi-Fi SSID (network name)"
);
pub(crate) const WIFI_PASSWORD: &str =
    str_from_env_or!("CONFIG_WIFI_PASSWORD", "test_password", "Wi-Fi password");
