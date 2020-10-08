#![no_std]

#[cfg(feature = "nrf52840dk")]
pub use nrf52840dk as board;

#[cfg(feature = "nucleo-f401re")]
pub use nucleo_f401re as board;

pub fn init() {
    board::init();
}
