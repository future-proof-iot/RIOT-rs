#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use riot_rs::{debug::println, embassy::network};

use embassy_net::udp::{PacketMetadata, UdpSocket};

// Moving work from https://github.com/embassy-rs/embassy/pull/2519 in here for the time being
mod udp_nal;
// Might warrant a standalone crate at some point
mod oluru;

mod seccontext;

#[riot_rs::task(autostart)]
async fn coap_run() {
    let stack = network::network_stack().await.unwrap();

    // FIXME trim to CoAP requirements
    let mut rx_meta = [PacketMetadata::EMPTY; 16];
    let mut rx_buffer = [0; 4096];
    let mut tx_meta = [PacketMetadata::EMPTY; 16];
    let mut tx_buffer = [0; 4096];

    let socket = UdpSocket::new(
        stack,
        &mut rx_meta,
        &mut rx_buffer,
        &mut tx_meta,
        &mut tx_buffer,
    );

    println!("Starting up CoAP server");

    // Can't that even bind to the Any address??
    // let local_any = "0.0.0.0:5683".parse().unwrap(); // shame
    let local_any = "10.42.0.61:5683".parse().unwrap(); // shame
    let unconnected = udp_nal::UnconnectedUdp::bind_multiple(socket, local_any)
        .await
        .unwrap();

    run(unconnected).await;
}

// FIXME: So far, this is necessary boiler plate; see ../README.md#networking for details
#[riot_rs::config(network)]
fn network_config() -> embassy_net::Config {
    use embassy_net::Ipv4Address;

    embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
        address: embassy_net::Ipv4Cidr::new(Ipv4Address::new(10, 42, 0, 61), 24),
        dns_servers: heapless::Vec::new(),
        gateway: Some(Ipv4Address::new(10, 42, 0, 1)),
    })
}

// Rest is from coap-message-demos/examples/std_embedded_nal_coap.rs

/// This function works on *any* UdpFullStack, including embedded ones -- only main() is what makes
/// this use POSIX sockets. (It does make use of a std based RNG, but that could be passed in just
/// as well for no_std operation).
async fn run<S>(mut sock: S)
where
    S: embedded_nal_async::UnconnectedUdp,
{
    use coap_handler_implementations::{HandlerBuilder, ReportingHandlerBuilder};

    let log = None;
    let buffer = scroll_ring::Buffer::<512>::default();
    // FIXME: Why doesn't scroll_ring provide that?
    struct Stdout<'a>(&'a scroll_ring::Buffer<512>);
    impl<'a> core::fmt::Write for Stdout<'a> {
        fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
            self.0.write(s.as_bytes());
            Ok(())
        }
    }
    let mut stdout = Stdout(&buffer);
    use core::fmt::Write;
    writeln!(stdout, "We have our own stdout now.").unwrap();
    writeln!(stdout, "With rings and atomics.").unwrap();

    use hexlit::hex;
    const R: &[u8] = &hex!("72cc4761dbd4c78f758931aa589d348d1ef874a7e303ede2f140dcf3e6aa4aac");
    let own_identity = (
        &lakers::CredentialRPK::new(lakers::EdhocMessageBuffer::new_from_slice(&hex!("A2026008A101A5010202410A2001215820BBC34960526EA4D32E940CAD2A234148DDC21791A12AFBCBAC93622046DD44F02258204519E257236B2A0CE2023F0931F1F386CA7AFDA64FCDE0108C224C51EABF6072")).expect("Credential should be small enough")).expect("Credential should be processable"),
        R,
        );

    let mut handler = coap_message_demos::full_application_tree(log)
        .at(
            &["stdout"],
            coap_scroll_ring_server::BufferHandler::new(&buffer),
        )
        .with_wkc();

    let mut handler = seccontext::OscoreEdhocHandler::new(own_identity, handler, stdout);

    println!("Server is ready.");

    let coap = embedded_nal_coap::CoAPShared::<3>::new();
    let (client, server) = coap.split();

    // going with an embassy_futures join instead of an async_std::task::spawn b/c CoAPShared is not
    // Sync, and async_std expects to work in multiple threads
    embassy_futures::join::join(
        async {
            server
                .run(&mut sock, &mut handler, &mut riot_rs::random::fast_rng())
                .await
                .expect("UDP error")
        },
        run_client_operations(client),
    )
    .await;
}

/// In parallel to server operation, this function performs some operations as a client.
///
/// This doubles as an experimentation ground for the client side of embedded_nal_coap and
/// coap-request in general.
async fn run_client_operations<const N: usize>(
    client: embedded_nal_coap::CoAPRuntimeClient<'_, N>,
) {
    // shame
    let demoserver = "10.42.0.1:1234".parse().unwrap();

    use coap_request::Stack;
    println!("Sending GET to {}...", demoserver);
    let response = client
        .to(demoserver)
        .request(
            coap_request_implementations::Code::get()
                .with_path("/other/separate")
                .processing_response_payload_through(|p| {
                    println!("Got payload {:?}", p);
                }),
        )
        .await;
    println!("Response {:?}", response);

    let req = coap_request_implementations::Code::post().with_path("/uppercase");

    println!("Sending POST...");
    let mut response = client.to(demoserver);
    let response = response.request(
        req.with_request_payload_slice(b"Set time to 1955-11-05")
            .processing_response_payload_through(|p| {
                println!("Uppercase is {}", core::str::from_utf8(p).unwrap())
            }),
    );
    let response = response.await;
    println!("Response {:?}", response);
}
