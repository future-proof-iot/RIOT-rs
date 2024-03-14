#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use riot_rs::{debug::println, embassy::network};

#[riot_rs::task(autostart)]
async fn udp_echo() {
    use embassy_net::udp::{PacketMetadata, UdpSocket};
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

        println!("Listening on UDP:1234...");
        if let Err(e) = socket.bind(1234) {
            println!("bind error: {:?}", e);
            continue;
        }

        loop {
            let (n, remote_endpoint) = match socket.recv_from(&mut buf).await {
                Ok((0, _)) => {
                    println!("read EOF");
                    break;
                }
                Ok((n, remote_endpoint)) => (n, remote_endpoint),
                Err(e) => {
                    println!("read error: {:?}", e);
                    break;
                }
            };

            println!("Received datagram from {:?}", remote_endpoint);

            //println!("rxd {:02x}", &buf[..n]);

            match socket.send_to(&buf[..n], remote_endpoint).await {
                Ok(()) => {}
                Err(e) => {
                    println!("write error: {:?}", e);
                    break;
                }
            };
        }
    }
}

#[riot_rs::config(network)]
fn network_config() -> embassy_net::Config {
    use embassy_net::Ipv4Address;

    embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
        address: embassy_net::Ipv4Cidr::new(Ipv4Address::new(10, 42, 0, 61), 24),
        dns_servers: heapless::Vec::new(),
        gateway: Some(Ipv4Address::new(10, 42, 0, 1)),
    })
}
