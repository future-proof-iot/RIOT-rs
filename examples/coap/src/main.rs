#![no_main]
#![no_std]
#![feature(impl_trait_in_assoc_type)]
#![feature(used_with_arg)]

use riot_rs::{debug::log::*, network};

// because coapcore depends on it temporarily
extern crate alloc;
use static_alloc::Bump;

#[global_allocator]
static A: Bump<[u8; 1 << 16]> = Bump::uninit();

#[riot_rs::task(autostart)]
async fn coap_run() {
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

    let handler = coap_message_demos::full_application_tree(log).at(
        &["stdout"],
        coap_scroll_ring_server::BufferHandler::new(&buffer),
    );

    let client_signal = embassy_sync::signal::Signal::new();

    // going with an embassy_futures join instead of RIOT-rs's spawn b/c CoAPShared is not Sync.
    embassy_futures::join::join(
        riot_rs::coap::coap_run(handler, stdout, &client_signal),
        run_client_operations(&client_signal),
    )
    .await;
}

/// In parallel to server operation, this function performs some operations as a client.
///
/// This doubles as an experimentation ground for the client side of embedded_nal_coap and
/// coap-request in general.
async fn run_client_operations(
    client_in: &embassy_sync::signal::Signal<
        embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
        &'static embedded_nal_coap::CoAPRuntimeClient<'static, 3>,
    >,
) {
    let client = client_in.wait().await;

    // shame
    let addr = "10.42.0.1:1234";
    let demoserver = addr.clone().parse().unwrap();

    use coap_request::Stack;
    info!("Sending GET to {}...", addr);
    let response = client
        .to(demoserver)
        .request(
            coap_request_implementations::Code::get()
                .with_path("/other/separate")
                .processing_response_payload_through(|p| {
                    info!("Got payload {:?}", p);
                }),
        )
        .await;
    info!("Response {:?}", response.map_err(|_| "TransportError"));

    let req = coap_request_implementations::Code::post().with_path("/uppercase");

    info!("Sending POST...");
    let mut response = client.to(demoserver);
    let response = response.request(
        req.with_request_payload_slice(b"Set time to 1955-11-05")
            .processing_response_payload_through(|p| {
                info!("Uppercase is {}", core::str::from_utf8(p).unwrap())
            }),
    );
    let response = response.await;
    info!("Response {:?}", response.map_err(|_| "TransportError"));
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
