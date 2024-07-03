//! A CoAP stack preconfigured for RIOT-rs
//! ======================================
//!
//! This crate mainly provides easy-to-use wrappers around the [`coapcore`] crate, with presets
//! tailored towards RIOT-rs: It utilizes [`embassy-net`] to open a network accessible CoAP socket,
//! [`riot-rs-random`] as a source of randomness, and [`lakers-crypto-rustcrypto`] for the
//! cryptographic algorithm implementations.
#![no_std]

// Moving work from https://github.com/embassy-rs/embassy/pull/2519 in here for the time being
mod udp_nal;

use riot_rs_embassy::embassy_net::udp::{PacketMetadata, UdpSocket};

use coapcore::seccontext;

pub async fn coap_task<const N: usize>(
    handler: impl coap_handler::Handler,
    client_runner: impl coapcore::ClientRunner<N>,
    logger: &mut impl core::fmt::Write,
) {
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

    // Can't that even bind to the Any address??
    // let local_any = "0.0.0.0:5683".parse().unwrap(); // shame
    let local_any = "10.42.0.61:5683".parse().unwrap(); // shame
    let mut sock = udp_nal::UnconnectedUdp::bind_multiple(socket, local_any)
        .await
        .unwrap();

    let mut rng = riot_rs_random::fast_rng();

    use hexlit::hex;
    const R: &[u8] = &hex!("72cc4761dbd4c78f758931aa589d348d1ef874a7e303ede2f140dcf3e6aa4aac");
    let own_identity = (
        &lakers::CredentialRPK::new(lakers::EdhocMessageBuffer::new_from_slice(&hex!("A2026008A101A5010202410A2001215820BBC34960526EA4D32E940CAD2A234148DDC21791A12AFBCBAC93622046DD44F02258204519E257236B2A0CE2023F0931F1F386CA7AFDA64FCDE0108C224C51EABF6072")).expect("Credential should be small enough")).expect("Credential should be processable"),
        R,
        );

    let mut handler = seccontext::OscoreEdhocHandler::new(own_identity, handler, logger, || {
        lakers_crypto_rustcrypto::Crypto::new(riot_rs_random::crypto_rng())
    });

    coapcore::coap_task(&mut sock, &mut handler, &mut rng, client_runner).await;
}
