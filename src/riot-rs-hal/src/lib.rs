//! This module dispatches between the ariel-os HAL crates.

#![no_std]

cfg_if::cfg_if! {
    if #[cfg(context = "nrf")] {
        pub use riot_rs_nrf::*;
    } else if #[cfg(context = "rp")] {
        pub use riot_rs_rp::*;
    } else if #[cfg(context = "esp")] {
        pub use riot_rs_esp::*;
    } else if #[cfg(context = "stm32")] {
        pub use riot_rs_stm32::*;
    } else if #[cfg(context = "ariel-os")] {
        compile_error!("this MCU family is not supported");
    } else {
        mod dummy;
        pub use dummy::*;
    }
}
