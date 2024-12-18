#![no_main]
#![no_std]
#![feature(impl_trait_in_assoc_type)]
#![feature(used_with_arg)]

use ariel_os::{
    debug::log::*,
    network,
    reexports::embassy_net::udp::{PacketMetadata, UdpSocket},
};

#[ariel_os::config(network)]
const NETWORK_CONFIG: ariel_os::reexports::embassy_net::Config = {
    use ariel_os::reexports::embassy_net::{self, Ipv4Address};

    embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
        address: embassy_net::Ipv4Cidr::new(Ipv4Address::new(10, 42, 0, 61), 24),
        dns_servers: heapless::Vec::new(),
        gateway: Some(Ipv4Address::new(10, 42, 0, 1)),
    })
};

#[ariel_os::task(autostart)]
async fn udp_echo() {
    let stack = network::network_stack().await.unwrap();

    let mut rx_meta = [PacketMetadata::EMPTY; 16];
    let mut rx_buffer = [0; 4096];
    let mut tx_meta = [PacketMetadata::EMPTY; 16];
    let mut tx_buffer = [0; 4096];
    let mut buf = [0; 4096];

    loop {
        let mut socket = UdpSocket::new(
            stack,
            &mut rx_meta,
            &mut rx_buffer,
            &mut tx_meta,
            &mut tx_buffer,
        );

        info!("Listening on UDP:1234...");
        if let Err(e) = socket.bind(1234) {
            info!("bind error: {:?}", e);
            continue;
        }

        loop {
            let (n, remote_endpoint) = match socket.recv_from(&mut buf).await {
                Ok((0, _)) => {
                    info!("read EOF");
                    break;
                }
                Ok((n, remote_endpoint)) => (n, remote_endpoint),
                Err(e) => {
                    info!("read error: {:?}", e);
                    break;
                }
            };

            info!("Received datagram from {:?}", remote_endpoint);

            match socket.send_to(&buf[..n], remote_endpoint).await {
                Ok(()) => {}
                Err(e) => {
                    info!("write error: {:?}", e);
                    break;
                }
            };
        }
    }
}
