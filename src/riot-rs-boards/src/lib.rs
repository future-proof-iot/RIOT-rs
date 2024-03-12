#![no_std]
#![feature(used_with_arg)]

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "nrf52dk")] {
        pub use nrf52dk as board;
    } else if #[cfg(feature = "dwm1001")] {
        pub use dwm1001 as board;
    } else if #[cfg(feature = "nrf52840dk")] {
        pub use nrf52840dk as board;
    } else if #[cfg(feature = "nrf52840-mdk")] {
        pub use nrf52840_mdk as board;
    } else if #[cfg(feature = "microbit")] {
        pub use microbit as board;
    } else if #[cfg(feature = "microbit-v2")] {
        pub use microbit_v2 as board;
    } else if #[cfg(feature = "nucleo-f401re")] {
        pub use nucleo_f401re as board;
    } else if #[cfg(feature = "rpi-pico")] {
        pub use rpi_pico as board;
    } else if #[cfg(feature = "rpi-pico-w")] {
        // sharing rpi-pico
        pub use rpi_pico as board;
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
