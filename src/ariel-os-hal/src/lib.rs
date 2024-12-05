//! Provides MCU-specific items.
//!
//! This module dispatches between one of the following crate, depending on the target MCU family:
//!
//! | Manufacturer         | MCU family  | Docs rendered for | Items imported                                       |
//! | -------------------- | ----------- | ----------------- | ---------------------------------------------------- |
//! | Espressif            | ESP32       | ESP32-C6          | [`ariel-os-esp::*`](../../ariel_os_esp/index.html)     |
//! | Nordic Semiconductor | nRF         | nRF52840          | [`ariel-os-nrf::*`](../../ariel_os_nrf/index.html)     |
//! | Raspberry Pi         | RP          | RP2040            | [`ariel-os-rp::*`](../../ariel_os_rp/index.html)       |
//! | STMicroelectronics   | STM32       | STM32W55RGVX      | [`ariel-os-stm32::*`](../../ariel_os_stm32/index.html) |
//!
//! Documentation is only rendered for the MCUs listed in the table above, but [many others are
//! supported](https://ariel-os.github.io/ariel-os/dev/docs/book/hardware_functionality_support.html).
//! To render the docs locally for the MCU of your choice, adapt [the `cargo doc` command used to
//! generate documentation for the relevant
//! crate](https://github.com/ariel-os/ariel-os/blob/main/.github/workflows/build-deploy-docs.yml).

#![no_std]
#![deny(clippy::pedantic)]

cfg_if::cfg_if! {
    if #[cfg(context = "nrf")] {
        pub use ariel_os_nrf::*;
    } else if #[cfg(context = "rp")] {
        pub use ariel_os_rp::*;
    } else if #[cfg(context = "esp")] {
        pub use ariel_os_esp::*;
    } else if #[cfg(context = "stm32")] {
        pub use ariel_os_stm32::*;
    } else if #[cfg(context = "ariel-os")] {
        compile_error!("this MCU family is not supported");
    } else {
        mod dummy;
        pub use dummy::*;
    }
}
