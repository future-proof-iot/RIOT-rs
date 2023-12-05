#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use riot_rs as _;

use riot_rs::embassy::TaskArgs;
use riot_rs::rt::debug::println;

#[embassy_executor::task]
async fn udp_echo(args: TaskArgs) {
    use embassy_net::udp::{UdpSocket, PacketMetadata};
    let stack = args.stack;

    let mut rx_meta = [PacketMetadata::EMPTY; 16];
    let mut rx_buffer = [0; 4096];
    let mut tx_meta = [PacketMetadata::EMPTY; 16];
    let mut tx_buffer = [0; 4096];
    let mut buf = [0; 4096];

    loop {
        let mut socket = UdpSocket::new(stack, &mut rx_meta, &mut rx_buffer, &mut tx_meta, &mut tx_buffer);

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

use linkme::distributed_slice;
use riot_rs::embassy::EMBASSY_TASKS;

#[distributed_slice(EMBASSY_TASKS)]
fn __start_udp_echo(spawner: embassy_executor::Spawner, t: TaskArgs) {
    spawner.spawn(udp_echo(t)).unwrap();
}

#[no_mangle]
fn riot_main() {
    println!(
        "Hello from riot_main()! Running on a {} board.",
        riot_rs::buildinfo::BOARD
    );

    loop {}
}
