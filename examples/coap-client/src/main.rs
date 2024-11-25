#![no_main]
#![no_std]
#![feature(impl_trait_in_assoc_type)]
#![feature(used_with_arg)]

use ariel_os::debug::log::info;

/// Run a CoAP stack without serving any actual resources.
#[ariel_os::task(autostart)]
async fn coap_run() {
    use coap_handler_implementations::new_dispatcher;

    let handler = new_dispatcher();
    ariel_os::coap::coap_run(handler).await;
}

#[ariel_os::task(autostart)]
async fn run_client_operations() {
    use coap_request::Stack;

    let client = ariel_os::coap::coap_client().await;

    // Corresponding to the fixed network setup, we select a fixed server address.
    let addr = "10.42.0.1:5683";
    let demoserver = addr.parse().unwrap();

    info!("Sending POST to {}...", demoserver);
    let request = coap_request_implementations::Code::post()
        .with_path("/uppercase")
        .with_request_payload_slice(b"This is Ariel OS")
        .processing_response_payload_through(|p| {
            info!(
                "Uppercase response is {}",
                core::str::from_utf8(p).map_err(|_| "not Unicode?")
            );
        });
    let response = client.to(demoserver).request(request).await;
    info!("Response {:?}", response.map_err(|_| "TransportError"));
}

// So far, this is necessary boilerplate; see ../../README.md#networking for details
#[ariel_os::config(network)]
fn network_config() -> ariel_os::reexports::embassy_net::Config {
    use ariel_os::reexports::embassy_net::{self, Ipv4Address};

    embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
        address: embassy_net::Ipv4Cidr::new(Ipv4Address::new(10, 42, 0, 61), 24),
        dns_servers: heapless::Vec::new(),
        gateway: Some(Ipv4Address::new(10, 42, 0, 1)),
    })
}
