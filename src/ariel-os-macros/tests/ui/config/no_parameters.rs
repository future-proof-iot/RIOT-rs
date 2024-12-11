// #![no_std]
#![no_main]

// This will not get used because the attribute macro is expected to fail
#[allow(unused_imports)]
use ariel_os::reexports::embassy_net;

// FAIL: this attribute macro requires parameters
#[ariel_os::config]
const NETWORK_CONFIG: embassy_net::Config = {
    use embassy_net::Ipv4Address;

    embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
        address: embassy_net::Ipv4Cidr::new(Ipv4Address::new(10, 42, 0, 61), 24),
        dns_servers: heapless::Vec::new(),
        gateway: Some(Ipv4Address::new(10, 42, 0, 1)),
    })
};
