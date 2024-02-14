//! Board specific configuration for the CYW43439 chip, found on the Raspberry Pi Pico W.

use cyw43_pio::PioSpi;
use embassy_rp::{bind_interrupts, pio::InterruptHandler};

use crate::{arch::peripherals, define_peripherals};

define_peripherals!(Cyw43Periphs {
    pwr: PIN_23,
    cs: PIN_25,
    pio: PIO0 = CYW43_PIO,
    dma: DMA_CH0 = CYW43_DMA_CH,
    dio: PIN_24,
    clk: PIN_29,
});

bind_interrupts!(pub struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<CYW43_PIO>;
});

pub type CywSpi = PioSpi<'static, CYW43_PIO, 0, CYW43_DMA_CH>;
