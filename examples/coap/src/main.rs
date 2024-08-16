#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use core::fmt::Write;

use riot_rs::embassy::embassy_net;

// because coapcore depends on it temporarily
extern crate alloc;
use static_alloc::Bump;

#[global_allocator]
static A: Bump<[u8; 1 << 16]> = Bump::uninit();

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

// This is adjusted from coap-message-demos/examples/std_embedded_nal_coap.rs

// FIXME: Why doesn't scroll_ring provide that?
#[derive(Clone)]
struct Stdout<'a, const N: usize>(&'a scroll_ring::Buffer<N>);
impl<'a, const N: usize> Write for Stdout<'a, N> {
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        self.0.write(s.as_bytes());
        Ok(())
    }
}

#[riot_rs::task(autostart)]
async fn run() {
    use coap_handler_implementations::{HandlerBuilder, ReportingHandlerBuilder};

    let log = None;
    let buffer = scroll_ring::Buffer::<512>::default();
    let mut stdout = Stdout(&buffer);
    writeln!(stdout, "We have our own stdout now.").unwrap();
    writeln!(stdout, "With rings and atomics.").unwrap();

    let handler = coap_message_demos::full_application_tree(log)
        .at(
            &["stdout"],
            coap_scroll_ring_server::BufferHandler::new(&buffer),
        )
        .with_wkc();

    writeln!(stdout, "Server is ready.").unwrap();

    riot_rs::coap::coap_task(handler, Client(stdout.clone()), &mut stdout).await;
}

struct Client<W: core::fmt::Write + Clone>(W);

impl<W: core::fmt::Write + Clone> Client<W> {
    async fn run_logging(
        mut self,
        client: embedded_nal_coap::CoAPRuntimeClient<'_, 3>,
    ) -> Result<(), &'static str> {
        // shame
        let demoserver = "10.42.0.1:1234"
            .parse()
            .map_err(|_| "Error parsing demo server address")?;

        use coap_request::Stack;
        writeln!(self.0, "Sending GET to {}...", demoserver).unwrap();

        let response = client
            .to(demoserver)
            .request(
                coap_request_implementations::Code::get()
                    .with_path("/other/separate")
                    .processing_response_payload_through(|p| {
                        writeln!(
                            self.0,
                            "Got payload {:?} length {}",
                            &p[..core::cmp::min(10, p.len())],
                            p.len()
                        )
                        .unwrap();
                    }),
            )
            .await
            .map_err(|_| "Error while trying to GET /other/separate")?;
        writeln!(self.0, "Response {:?}", response).unwrap();

        Ok(())
    }
}

impl<W: core::fmt::Write + Clone> coapcore::ClientRunner<3> for Client<W> {
    /// In parallel to server operation, this function performs some operations as a client.
    ///
    /// This doubles as an experimentation ground for the client side of embedded_nal_coap and
    /// coap-request in general.
    async fn run(self, client: embedded_nal_coap::CoAPRuntimeClient<'_, 3>) {
        let mut stdout = self.0.clone();
        match self.run_logging(client).await {
            Ok(_) => writeln!(stdout, "Client process completed").unwrap(),
            Err(e) => writeln!(stdout, "Client process erred out: {e}").unwrap(),
        }
    }
}
