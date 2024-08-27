// #![no_std]
#![no_main]

// This will not get used because the attribute macro is expected to fail
#[allow(unused_imports)]
use riot_rs::embassy_net;

// FAIL: network and usb cannot be used at the same time
#[riot_rs::config(network, usb)]
fn network_config() -> embassy_net::Config {
    use embassy_net::Ipv4Address;

    embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
        address: embassy_net::Ipv4Cidr::new(Ipv4Address::new(10, 42, 0, 61), 24),
        dns_servers: heapless::Vec::new(),
        gateway: Some(Ipv4Address::new(10, 42, 0, 1)),
    })
}

