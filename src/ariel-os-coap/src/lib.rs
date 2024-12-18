//! A CoAP stack preconfigured for Ariel OS.
//!
//! This crate mainly provides easy-to-use wrappers around the [`coapcore`] crate, with presets
//! tailored towards Ariel OS: It utilizes [`embassy_net`] to open a network accessible CoAP socket
//! and selects [`embedded_nal_coap`] for CoAP over UDP, it selects [`ariel_os_random`] as a source
//! of randomness, and [`lakers_crypto_rustcrypto`] for the cryptographic algorithm
//! implementations.
#![no_std]
#![deny(missing_docs)]
#![deny(clippy::pedantic)]

// Moving work from https://github.com/embassy-rs/embassy/pull/2519 in here for the time being
mod udp_nal;

use ariel_os_debug::log::info;
use ariel_os_embassy::sendcell::SendCell;
use coap_handler_implementations::ReportingHandlerBuilder;
use coapcore::seccontext;
use embassy_net::udp::{PacketMetadata, UdpSocket};
use embassy_sync::once_lock::OnceLock;
use static_cell::StaticCell;

const CONCURRENT_REQUESTS: usize = 3;

static CLIENT: OnceLock<
    SendCell<embedded_nal_coap::CoAPRuntimeClient<'static, CONCURRENT_REQUESTS>>,
> = OnceLock::new();

/// Runs a CoAP server with the given handler on the system's CoAP transports.
///
/// As the CoAP stack gets ready, it also unblocks [`coap_client`].
///
/// # Panics
///
/// This can only be run once, as it sets up a system wide CoAP handler.
pub async fn coap_run(handler: impl coap_handler::Handler + coap_handler::Reporting) -> ! {
    use hexlit::hex;
    const R: [u8; 32] = hex!("72cc4761dbd4c78f758931aa589d348d1ef874a7e303ede2f140dcf3e6aa4aac");

    static COAP: StaticCell<embedded_nal_coap::CoAPShared<CONCURRENT_REQUESTS>> = StaticCell::new();

    let stack = ariel_os_embassy::network::network_stack().await.unwrap();

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

    info!("Starting up CoAP server");

    // Can't that even bind to the Any address??
    // let local_any = "0.0.0.0:5683".parse().unwrap(); // shame
    let local_any = "10.42.0.61:5683".parse().unwrap(); // shame
    let mut unconnected = udp_nal::UnconnectedUdp::bind_multiple(socket, local_any)
        .await
        .unwrap();

    let own_identity = (
        &lakers::Credential::parse_ccs(&hex!("A2026008A101A5010202410A2001215820BBC34960526EA4D32E940CAD2A234148DDC21791A12AFBCBAC93622046DD44F02258204519E257236B2A0CE2023F0931F1F386CA7AFDA64FCDE0108C224C51EABF6072")).expect("Credential should be processable"),
        &R,
        );

    // FIXME: Should we allow users to override that? After all, this is just convenience and may
    // be limiting in special applications.
    let handler = handler.with_wkc();
    let mut handler = seccontext::OscoreEdhocHandler::new(own_identity, handler, || {
        lakers_crypto_rustcrypto::Crypto::new(ariel_os_random::crypto_rng())
    });

    info!("Server is ready.");

    let coap = COAP.init_with(embedded_nal_coap::CoAPShared::new);
    let (client, server) = coap.split();
    CLIENT
        .init(SendCell::new_async(client).await)
        .ok()
        .expect("CLIENT can not be populated when COAP was just not populated.");

    server
        .run(
            &mut unconnected,
            &mut handler,
            &mut ariel_os_random::fast_rng(),
        )
        .await
        .expect("UDP error");
    unreachable!("embassy-net's sockets do not get closed (but embedded-nal-coap can't know that)");
}

/// Returns a CoAP client requester.
///
/// This asynchronously blocks until [`coap_run`] has been called, and the CoAP stack is
/// operational.
///
/// # Panics
///
/// This is currently only available from the thread that hosts the network stack, and panics
/// otherwise. This restriction will be lifted in the future (by generalization in
/// [`embedded_nal_coap`] to allow different mutexes).
pub async fn coap_client(
) -> &'static embedded_nal_coap::CoAPRuntimeClient<'static, CONCURRENT_REQUESTS> {
    CLIENT
        .get()
        .await
        .get_async()
        .await // Not an actual await, just a convenient way to see which executor is running
        .expect("CoAP client can currently only be used from the thread the network is bound to")
}
