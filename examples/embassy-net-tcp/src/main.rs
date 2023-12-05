#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use riot_rs as _;

use riot_rs::embassy::TaskArgs;
use riot_rs::rt::debug::println;

use embedded_io_async::Write;

#[embassy_executor::task]
async fn tcp_echo(args: TaskArgs) {
    use embassy_net::tcp::TcpSocket;
    let stack = args.stack;

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

use linkme::distributed_slice;
use riot_rs::embassy::EMBASSY_TASKS;

#[distributed_slice(EMBASSY_TASKS)]
fn __start_tcp_echo(spawner: embassy_executor::Spawner, t: TaskArgs) {
    spawner.spawn(tcp_echo(t)).unwrap();
}

#[no_mangle]
fn riot_main() {
    println!(
        "Hello from riot_main()! Running on a {} board.",
        riot_rs::buildinfo::BOARD
    );

    loop {}
}
