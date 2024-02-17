#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use riot_rs::embassy::network;
use riot_rs::rt::debug::println;

use embedded_io_async::Write;

#[embassy_executor::task]
async fn tcp_echo() {
    use embassy_net::tcp::TcpSocket;
    let stack = network::network_stack().await.unwrap();

    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    let mut buf = [0; 4096];

    loop {
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(embassy_time::Duration::from_secs(10)));

        println!("Listening on TCP:1234...");
        if let Err(e) = socket.accept(1234).await {
            println!("accept error: {:?}", e);
            continue;
        }

        println!("Received connection from {:?}", socket.remote_endpoint());

        loop {
            let n = match socket.read(&mut buf).await {
                Ok(0) => {
                    println!("read EOF");
                    break;
                }
                Ok(n) => n,
                Err(e) => {
                    println!("read error: {:?}", e);
                    break;
                }
            };

            //println!("rxd {:02x}", &buf[..n]);

            match socket.write_all(&buf[..n]).await {
                Ok(()) => {}
                Err(e) => {
                    println!("write error: {:?}", e);
                    break;
                }
            };
        }
    }
}

// TODO: macro up this
use riot_rs::embassy::{arch::OptionalPeripherals, Spawner};
#[riot_rs::embassy::distributed_slice(riot_rs::embassy::EMBASSY_TASKS)]
#[linkme(crate = riot_rs::embassy::linkme)]
fn __init_tcp_echo(spawner: &Spawner, _peripherals: &mut OptionalPeripherals) {
    spawner.spawn(tcp_echo()).unwrap();
}

#[no_mangle]
fn riot_rs_network_config() -> embassy_net::Config {
    use embassy_net::Ipv4Address;

    embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
        address: embassy_net::Ipv4Cidr::new(Ipv4Address::new(10, 42, 0, 61), 24),
        dns_servers: heapless::Vec::new(),
        gateway: Some(Ipv4Address::new(10, 42, 0, 1)),
    })
}
