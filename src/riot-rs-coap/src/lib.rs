//! A CoAP stack preconfigured for RIOT-rs.
//!
//! This crate mainly provides easy-to-use wrappers around the [`coapcore`] crate, with presets
//! tailored towards RIOT-rs: It utilizes [`embassy_net`] to open a network accessible CoAP socket
//! and selects [`embedded_nal_coap`] for CoAP over UDP, it selects [`riot_rs_random`] as a source
//! of randomness, and [`lakers_crypto_rustcrypto`] for the cryptographic algorithm
//! implementations.
#![no_std]
#![deny(missing_docs)]
#![deny(clippy::pedantic)]

// Moving work from https://github.com/embassy-rs/embassy/pull/2519 in here for the time being
mod udp_nal;

use coap_handler_implementations::{HandlerBuilder, ReportingHandlerBuilder};
use coapcore::seccontext;
use critical_section::Mutex;
use embassy_net::udp::{PacketMetadata, UdpSocket};
use riot_rs_debug::log::*;
use static_cell::StaticCell;

const CONCURRENT_REQUESTS: usize = 3;

// FIXME: log_stdout is not something we want to have here
// FIXME: I'd rather have the client_out available anywhere, but at least the way CoAPRuntimeClient
// is set up right now, server and client have to run in the same thread.
/// Run a CoAP server with the given handler on the system's CoAP transports.
pub async fn coap_run(
    handler: impl coap_handler::Handler + coap_handler::Reporting,
    client_out: &embassy_sync::signal::Signal<
        embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
        &'static embedded_nal_coap::CoAPRuntimeClient<'static, CONCURRENT_REQUESTS>,
    >,
) -> ! {
    let stack = riot_rs_embassy::network::network_stack().await.unwrap();

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
    use embedded_nal_async::UnconnectedUdp;

    info!("Starting up CoAP server");

    // Can't that even bind to the Any address??
    // let local_any = "0.0.0.0:5683".parse().unwrap(); // shame
    let local_any = "10.42.0.61:5683".parse().unwrap(); // shame
    let mut unconnected = udp_nal::UnconnectedUdp::bind_multiple(socket, local_any)
        .await
        .unwrap();

    use hexlit::hex;
    const R: &[u8] = &hex!("72cc4761dbd4c78f758931aa589d348d1ef874a7e303ede2f140dcf3e6aa4aac");
    let own_identity = (
        &lakers::CredentialRPK::new(lakers::EdhocMessageBuffer::new_from_slice(&hex!("A2026008A101A5010202410A2001215820BBC34960526EA4D32E940CAD2A234148DDC21791A12AFBCBAC93622046DD44F02258204519E257236B2A0CE2023F0931F1F386CA7AFDA64FCDE0108C224C51EABF6072")).expect("Credential should be small enough")).expect("Credential should be processable"),
        R,
        );

    // FIXME: Should we allow users to override that? After all, this is just convenience and may
    // be limiting in special applications.
    let handler = handler.with_wkc();
    let mut handler = seccontext::OscoreEdhocHandler::new(own_identity, handler, || {
        lakers_crypto_rustcrypto::Crypto::new(riot_rs_random::crypto_rng())
    });

    info!("Server is ready.");

    static COAP: StaticCell<embedded_nal_coap::CoAPShared<CONCURRENT_REQUESTS>> = StaticCell::new();
    let coap = COAP.init_with(|| embedded_nal_coap::CoAPShared::new());
    static CLIENT: StaticCell<embedded_nal_coap::CoAPRuntimeClient<'static, CONCURRENT_REQUESTS>> =
        StaticCell::new();
    let (client, server) = coap.split();
    let client = CLIENT.init_with(|| client);

    client_out.signal(client);

    server
        .run(
            &mut unconnected,
            &mut handler,
            &mut riot_rs_random::fast_rng(),
        )
        .await
        .expect("UDP error");
    unreachable!("embassy-net's sockets do not get closed (but embedded-nal-coap can't know that)");
}
