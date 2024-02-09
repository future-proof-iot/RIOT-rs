//! To provide a custom USB configuration, use the `riot_rs::config` attribute macro.

pub use crate::arch::usb::UsbDriver;

pub type UsbBuilder = embassy_usb::Builder<'static, UsbDriver>;

#[linkme::distributed_slice]
pub static USB_BUILDER_HOOKS: [&crate::delegate::Delegate<UsbBuilder>] = [..];

#[embassy_executor::task]
pub(crate) async fn usb_task(mut device: embassy_usb::UsbDevice<'static, UsbDriver>) -> ! {
    device.run().await
}

#[cfg(feature = "usb-ethernet")]
pub(crate) mod ethernet {
    use embassy_usb::class::cdc_ncm::embassy_net::{Device, Runner};

    use crate::{arch::usb::UsbDriver, network::ETHERNET_MTU};

    pub type NetworkDevice = Device<'static, ETHERNET_MTU>;

    #[embassy_executor::task]
    pub async fn usb_ncm_task(class: Runner<'static, UsbDriver, ETHERNET_MTU>) -> ! {
        class.run().await
    }
}

pub(crate) fn config() -> embassy_usb::Config<'static> {
    #[cfg(not(feature = "override-usb-config"))]
    {
        // Create embassy-usb Config
        let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
        config.manufacturer = Some("Embassy");
        config.product = Some("USB-Ethernet example");
        config.serial_number = Some("12345678");
        config.max_power = 100;
        config.max_packet_size_0 = 64;

        // Required for Windows support.
        config.composite_with_iads = true;
        config.device_class = 0xEF;
        config.device_sub_class = 0x02;
        config.device_protocol = 0x01;
        config
    }
    #[cfg(feature = "override-usb-config")]
    {
        extern "Rust" {
            fn riot_rs_usb_config() -> embassy_usb::Config<'static>;
        }
        unsafe { riot_rs_usb_config() }
    }
}
