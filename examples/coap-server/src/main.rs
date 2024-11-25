#![no_main]
#![no_std]
#![feature(impl_trait_in_assoc_type)]
#![feature(used_with_arg)]

/// Due to mismatches between CoAP components, coapcore currently needs an allocator. This example
/// provides the one that can be made most easily.
mod alloc {
    extern crate alloc;

    #[global_allocator]
    static A: static_alloc::Bump<[u8; 1 << 16]> = static_alloc::Bump::uninit();
}

#[ariel_os::task(autostart)]
async fn coap_run() {
    use coap_handler_implementations::{
        new_dispatcher, HandlerBuilder, ReportingHandlerBuilder, SimpleRendered,
    };

    let handler = new_dispatcher()
        // We offer a single resource: /hello, which responds just with a text string.
        .at(&["hello"], SimpleRendered("Hello from Ariel OS"))
        .with_wkc();

    ariel_os::coap::coap_run(handler).await;
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
