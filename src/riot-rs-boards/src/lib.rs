#![no_std]
#![feature(used_with_arg)]

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "ai-c3")] {
        pub use ai_c3 as board;
    } else if #[cfg(feature = "espressif-esp32-c3-lcdkit")] {
        pub use espressif_esp32_c3_lcdkit as board;
    } else if #[cfg(feature = "espressif-esp32-c6-devkitc-1")] {
        pub use espressif_esp32_c6_devkitc_1 as board;
    } else if #[cfg(feature = "espressif-esp32-s3-wroom-1")] {
        pub use espressif_esp32_s3_wroom_1 as board;
    } else if #[cfg(feature = "nrf52dk")] {
        pub use nrf52dk as board;
    } else if #[cfg(feature = "dwm1001")] {
        pub use dwm1001 as board;
    } else if #[cfg(feature = "nrf52840dk")] {
        pub use nrf52840dk as board;
    } else if #[cfg(feature = "nrf52840-mdk")] {
        pub use nrf52840_mdk as board;
    } else if #[cfg(feature = "nrf5340dk")] {
        pub use nrf5340dk as board;
    } else if #[cfg(feature = "microbit")] {
        pub use microbit as board;
    } else if #[cfg(feature = "microbit-v2")] {
        pub use microbit_v2 as board;
    } else if #[cfg(feature = "nucleo-f401re")] {
        pub use nucleo_f401re as board;
    } else if #[cfg(feature = "particle-xenon")] {
        pub use particle_xenon as board;
    } else if #[cfg(feature = "rpi-pico")] {
        pub use rpi_pico as board;
    } else if #[cfg(feature = "rpi-pico-w")] {
        // sharing rpi-pico
        pub use rpi_pico as board;
    } else if #[cfg(feature = "st-nucleo-f401re")] {
        pub use st_nucleo_f401re as board;
    } else if #[cfg(feature = "st-nucleo-wb55")] {
        pub use st_nucleo_wb55 as board;
    } else if #[cfg(feature = "st-nucleo-h755zi-q")] {
        pub use st_nucleo_h755zi_q as board;
    } else if #[cfg(feature = "no-boards")] {
        // Do nothing
    } else {
        compile_error!("no board feature selected");
    }
}

#[cfg(not(feature = "no-boards"))]
#[linkme::distributed_slice(riot_rs_rt::INIT_FUNCS)]
fn init() {
    board::init();
}
