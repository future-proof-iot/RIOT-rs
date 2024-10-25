#[cfg(feature = "wifi-cyw43")]
pub(crate) use crate::arch::cyw43::NetworkDevice;

#[cfg(feature = "wifi-esp")]
pub(crate) use crate::arch::wifi::esp_wifi::NetworkDevice;
