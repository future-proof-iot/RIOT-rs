#[cfg(feature = "wifi-cyw43")]
pub(crate) use crate::hal::cyw43::NetworkDevice;

#[cfg(feature = "wifi-esp")]
pub(crate) use crate::hal::wifi::esp_wifi::NetworkDevice;
