#[cfg_attr(builder = "rpi-pico-w", path = "cyw43/rpi-pico-w.rs")]
mod rpi_pico_w;

use cyw43::{Control, Runner};
use embassy_executor::Spawner;
use embassy_rp::{
    gpio::{Level, Output},
    pio::Pio,
};
use riot_rs_debug::log::info;
use rpi_pico_w::{CywSpi, Irqs};
use static_cell::StaticCell;

pub type NetworkDevice = cyw43::NetDriver<'static>;

pub async fn join(mut control: cyw43::Control<'static>) {
    loop {
        //control.join_open(WIFI_NETWORK).await;
        match control
            .join_wpa2(crate::wifi::WIFI_NETWORK, crate::wifi::WIFI_PASSWORD)
            .await
        {
            Ok(_) => {
                info!("Wifi connected!");
                break;
            }
            Err(err) => {
                info!(" Wifi join failed with status={}", err.status);
            }
        }
    }
}

#[embassy_executor::task]
async fn wifi_cyw43_task(runner: Runner<'static, Output<'static>, CywSpi>) -> ! {
    runner.run().await
}

pub async fn device<'a, 'b: 'a>(
    peripherals: &'a mut crate::OptionalPeripherals,
    spawner: &Spawner,
) -> (embassy_net_driver_channel::Device<'b, 1514>, Control<'b>) {
    let pins = rpi_pico_w::take_pins(peripherals);

    let fw = include_bytes!("cyw43/firmware/43439A0.bin");
    let clm = include_bytes!("cyw43/firmware/43439A0_clm.bin");

    // To make flashing faster for development, you may want to flash the firmwares independently
    // at hardcoded addresses, instead of baking them into the program with `include_bytes!`:
    //     probe-rs download 43439A0.bin --format bin --chip RP2040 --base-address 0x10100000
    //     probe-rs download 43439A0_clm.bin --format bin --chip RP2040 --base-address 0x10140000
    //let fw = unsafe { core::slice::from_raw_parts(0x10100000 as *const u8, 230321) };
    //let clm = unsafe { core::slice::from_raw_parts(0x10140000 as *const u8, 4752) };

    let pwr = Output::new(pins.pwr, Level::Low);
    let cs = Output::new(pins.cs, Level::High);
    let mut pio = Pio::new(pins.pio, Irqs);
    let spi = CywSpi::new(
        &mut pio.common,
        pio.sm0,
        pio.irq0,
        cs,
        pins.dio,
        pins.clk,
        pins.dma,
    );

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let (net_device, mut control, runner) =
        cyw43::new(STATE.init_with(|| cyw43::State::new()), pwr, spi, fw).await;

    // this needs to be spawned here (before using `control`)
    spawner.spawn(wifi_cyw43_task(runner)).unwrap();

    control.init(clm).await;

    // control
    //     .set_power_management(cyw43::PowerManagementMode::PowerSave)
    //     .await;

    (net_device, control)
}
