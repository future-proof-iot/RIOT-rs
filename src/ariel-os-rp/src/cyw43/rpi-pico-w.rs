//! Board specific configuration for the CYW43439 chip, found on the Raspberry Pi Pico W.

use cyw43_pio::PioSpi;
use embassy_rp::{bind_interrupts, peripherals, pio::InterruptHandler};

bind_interrupts!(pub struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<CYW43_PIO>;
});

pub type CywSpi = PioSpi<'static, CYW43_PIO, 0, CYW43_DMA_CH>;

pub struct Cyw43Periphs {
    pub pwr: peripherals::PIN_23,
    pub cs: peripherals::PIN_25,
    pub pio: peripherals::PIO0,
    pub dma: peripherals::DMA_CH0,
    pub dio: peripherals::PIN_24,
    pub clk: peripherals::PIN_29,
}

#[expect(non_camel_case_types)]
type CYW43_PIO = peripherals::PIO0;
#[expect(non_camel_case_types)]
type CYW43_DMA_CH = peripherals::DMA_CH0;

pub fn take_pins(peripherals: &mut crate::OptionalPeripherals) -> Cyw43Periphs {
    Cyw43Periphs {
        pwr: peripherals.PIN_23.take().unwrap(),
        cs: peripherals.PIN_25.take().unwrap(),
        pio: peripherals.PIO0.take().unwrap(),
        dma: peripherals.DMA_CH0.take().unwrap(),
        dio: peripherals.PIN_24.take().unwrap(),
        clk: peripherals.PIN_29.take().unwrap(),
    }
}
