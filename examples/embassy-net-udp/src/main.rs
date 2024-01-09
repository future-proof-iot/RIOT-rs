#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use riot_rs::embassy::{arch, Application, ApplicationInitError, Drivers, InitializationArgs};

use riot_rs::rt::debug::println;

#[embassy_executor::task]
async fn udp_echo(drivers: Drivers) {
    use embassy_net::udp::{PacketMetadata, UdpSocket};
    let stack = drivers.stack.get().unwrap();

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

struct UdpEcho {}

impl Application for UdpEcho {
    fn initialize(
        _peripherals: &mut arch::OptionalPeripherals,
        _init_args: InitializationArgs,
    ) -> Result<&dyn Application, ApplicationInitError> {
        Ok(&Self {})
    }

    fn start(&self, spawner: embassy_executor::Spawner, drivers: Drivers) {
        spawner.spawn(udp_echo(drivers)).unwrap();
    }
}

riot_rs::embassy::riot_initialize!(UdpEcho);

#[no_mangle]
fn riot_main() {
    println!(
        "Hello from riot_main()! Running on a {} board.",
        riot_rs::buildinfo::BOARD
    );

    loop {}
}
