#[cfg(feature = "wifi-cyw43")]
pub(crate) use riot_rs_rp::cyw43::NetworkDevice;

#[cfg(feature = "wifi-esp")]
pub(crate) use riot_rs_esp::wifi::esp_wifi::NetworkDevice;
