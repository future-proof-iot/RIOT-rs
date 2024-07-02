//! A CoAP stack for embedded devices with built-in OSCORE/EDHOC support
//! ====================================================================
//!
//! This crate provides an asynchronous task that serves CoAP requests on a UDP port provided by
//! the application as an `embedded-nal` socket, and processes CoAP along with its security
//! components OSCORE and EDHOC before passing on authorized requests to the application.
//!
//! The crate is under heavy development.
#![no_std]
#![feature(lint_reasons)]

// Might warrant a standalone crate at some point
//
// This is pub only to make the doctests run (but the crate's pub-ness needs a major overhaul
// anyway)
pub mod oluru;
pub mod seccontext;

// This is a trait because I haven't managed to work out any lifetimes for a single client
// callback; tried
// pub async fn coap_task<ClientOps, ClientFuture>(
//     sock: &mut impl embedded_nal_async::UnconnectedUdp,
//     handler: &mut impl coap_handler::Handler,
//     rng: &mut impl rand_core::RngCore,
//     run_client_operations: ClientOps,
// ) where
//     for<'a> ClientOps: Fn(embedded_nal_coap::CoAPRuntimeClient<'a, 3>) -> ClientFuture,
//     ClientFuture: core::future::Future<Output = ()>,
// {
// which typechecks on its own, but won't take a static async fn (or closure that returns an async
// block) as input.
pub trait ClientRunner<const N: usize> {
    #[allow(async_fn_in_trait, reason = "We explicitly expect this to not be send")]
    async fn run(self, client: embedded_nal_coap::CoAPRuntimeClient<'_, N>);
}

pub async fn coap_task<const N: usize>(
    sock: &mut impl embedded_nal_async::UnconnectedUdp,
    handler: &mut impl coap_handler::Handler,
    rng: &mut impl rand_core::RngCore,
    client_runner: impl ClientRunner<N>,
) {
    let coap = embedded_nal_coap::CoAPShared::<N>::new();
    let (client, server) = coap.split();

    // going with an embassy_futures join instead of an async_std::task::spawn b/c CoAPShared is not
    // Sync, and async_std expects to work in multiple threads
    embassy_futures::join::join(
        async { server.run(sock, handler, rng).await.expect("UDP error") },
        client_runner.run(client),
    )
    .await;
}
