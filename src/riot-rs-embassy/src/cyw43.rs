pub use cyw43::NetDriver;

use cyw43::{Control, Runner};
use cyw43_pio::PioSpi;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::pio::{InterruptHandler, Pio};

use crate::{arch::peripherals, arch::OptionalPeripherals, define_peripherals, make_static};

// board specifics start
define_peripherals!(Cyw43Periphs {
    pwr: PIN_23,
    cs: PIN_25,
    pio: PIO0 = CYW43_PIO,
    dma: DMA_CH0 = CYW43_DMA_CH,
    dio: PIN_24,
    clk: PIN_29,
});

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<CYW43_PIO>;
});

type CywSpi = PioSpi<'static, CYW43_PIO, 0, CYW43_DMA_CH>;
// board specifics end

#[embassy_executor::task]
async fn wifi_cyw43_task(runner: Runner<'static, Output<'static>, CywSpi>) -> ! {
    runner.run().await
}

pub(crate) async fn device<'a, 'b: 'a>(
    p: &'a mut OptionalPeripherals,
    spawner: &crate::Spawner,
) -> (embassy_net_driver_channel::Device<'b, 1514>, Control<'b>) {
    let p = Cyw43Periphs::take_from(p).unwrap();

    let fw = include_bytes!("../firmware/cyw43/43439A0.bin");
    let clm = include_bytes!("../firmware/cyw43/43439A0_clm.bin");

    // To make flashing faster for development, you may want to flash the firmwares independently
    // at hardcoded addresses, instead of baking them into the program with `include_bytes!`:
    //     probe-rs download 43439A0.bin --format bin --chip RP2040 --base-address 0x10100000
    //     probe-rs download 43439A0_clm.bin --format bin --chip RP2040 --base-address 0x10140000
    //let fw = unsafe { core::slice::from_raw_parts(0x10100000 as *const u8, 230321) };
    //let clm = unsafe { core::slice::from_raw_parts(0x10140000 as *const u8, 4752) };

    let pwr = Output::new(p.pwr, Level::Low);
    let cs = Output::new(p.cs, Level::High);
    let mut pio = Pio::new(p.pio, Irqs);
    let spi = PioSpi::new(&mut pio.common, pio.sm0, pio.irq0, cs, p.dio, p.clk, p.dma);

    let state = make_static!(cyw43::State::new());
    let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;

    // this needs to be spawned here (before using `control`)
    spawner.spawn(wifi_cyw43_task(runner)).unwrap();

    control.init(clm).await;

    // control
    //     .set_power_management(cyw43::PowerManagementMode::PowerSave)
    //     .await;

    (net_device, control)
}
