#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]
#![feature(used_with_arg)]

use ariel_os::{
    debug::log::info,
    reexports::embassy_usb,
    usb::{UsbBuilderHook, UsbDriver},
    StaticCell,
};
use embassy_usb::{
    class::cdc_acm::{CdcAcmClass, State},
    driver::EndpointError,
};

const MAX_FULL_SPEED_PACKET_SIZE: u8 = 64;

#[ariel_os::config(usb)]
const USB_CONFIG: ariel_os::reexports::embassy_usb::Config = {
    let mut config = ariel_os::reexports::embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Ariel OS");
    config.product = Some("USB serial example");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = MAX_FULL_SPEED_PACKET_SIZE;

    // Required for Windows support.
    config.composite_with_iads = true;
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config
};

#[ariel_os::task(autostart, usb_builder_hook)]
async fn main() {
    info!("Hello World!");

    static STATE: StaticCell<State> = StaticCell::new();

    // Create and inject the USB class on the system USB builder.
    let mut class = USB_BUILDER_HOOK
        .with(|builder| {
            CdcAcmClass::new(
                builder,
                STATE.init_with(|| State::new()),
                MAX_FULL_SPEED_PACKET_SIZE.into(),
            )
        })
        .await;

    // Do stuff with the class!
    loop {
        class.wait_connection().await;
        info!("Connected");
        let _ = echo(&mut class).await;
        info!("Disconnected");
    }
}

struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected {},
        }
    }
}

async fn echo(class: &mut CdcAcmClass<'static, UsbDriver>) -> Result<(), Disconnected> {
    let mut buf = [0; MAX_FULL_SPEED_PACKET_SIZE as usize];
    loop {
        let n = class.read_packet(&mut buf).await?;
        let data = &buf[..n];
        info!("data: {:x}", data);
        class.write_packet(data).await?;
    }
}
