#![no_main]
#![no_std]
#![feature(impl_trait_in_assoc_type)]
#![feature(used_with_arg)]
#![feature(noop_waker)]

use riot_rs::debug::log::{error, info};
use riot_rs::storage;

// because coapcore depends on it temporarily
extern crate alloc;
use static_alloc::Bump;

#[global_allocator]
static A: Bump<[u8; 1 << 16]> = Bump::uninit();

/// Represents a concrete key inside the sytem wide storage as a CBOR resource that has a single
/// text value. `T` must practically be both serde for Postcard, and … FIXME: through which CBOR
/// handler does that even go? At any rate, `heapless::String<64>` is a suitable type.
struct CborStorageAccess<T> {
    key: &'static str,
    _phantom: core::marker::PhantomData<T>,
}

impl<T> CborStorageAccess<T> {
    fn new(key: &'static str) -> Self {
        Self {
            key,
            _phantom: Default::default(),
        }
    }
}

impl<T> coap_handler_implementations::TypeRenderable for CborStorageAccess<T>
where
    T: serde::ser::Serialize,
    T: for<'de> serde::de::Deserialize<'de>,
    T: Clone, // put gives us an &, but to store, we need to move one in
{
    type Get = T;
    type Post = ();
    type Put = T;

    fn get(&mut self) -> Result<Self::Get, u8> {
        use core::future::Future;
        let future = core::pin::pin!(storage::get(self.key));
        // Let's hope this is not *really* blocking
        match future.poll(&mut core::task::Context::from_waker(
            &core::task::Waker::noop(),
        )) {
            core::task::Poll::Ready(data) => data
                .map_err(|_| {
                    error!("Reading data failed"); // sadly, the error is not Format
                    coap_numbers::code::BAD_REQUEST
                })
                .and_then(|data| data.ok_or(coap_numbers::code::NOT_FOUND)),
            _ => {
                error!("Data was not available for reading instantly.");
                Err(coap_numbers::code::INTERNAL_SERVER_ERROR)
            }
        }
    }

    fn put(&mut self, value: &Self::Put) -> u8 {
        use core::future::Future;
        let future = core::pin::pin!(storage::insert(self.key, value.clone()));
        // Let's hope this is not *really* blocking
        match future.poll(&mut core::task::Context::from_waker(
            &core::task::Waker::noop(),
        )) {
            core::task::Poll::Ready(Ok(())) => coap_numbers::code::CHANGED,
            // more like "some other error"
            core::task::Poll::Ready(Err(_)) => {
                error!("Storing data failed"); // sadly, the error is not Format
                coap_numbers::code::BAD_REQUEST
            }
            _ => {
                error!("Data was not written instantly.");
                coap_numbers::code::INTERNAL_SERVER_ERROR
            }
        }
    }

    fn delete(&mut self) -> u8 {
        use core::future::Future;
        let future = core::pin::pin!(storage::remove(self.key));
        // Let's hope this is not *really* blocking
        match future.poll(&mut core::task::Context::from_waker(
            &core::task::Waker::noop(),
        )) {
            core::task::Poll::Ready(Ok(())) => coap_numbers::code::DELETED,
            // more like "some other error"
            core::task::Poll::Ready(Err(_)) => {
                error!("Storing data failed"); // sadly, the error is not Format
                coap_numbers::code::BAD_REQUEST
            }
            _ => {
                error!("Data was not written instantly.");
                coap_numbers::code::INTERNAL_SERVER_ERROR
            }
        }
    }
}

#[riot_rs::task(autostart)]
async fn coap_run() {
    use coap_handler_implementations::HandlerBuilder;

    type MyConfig = heapless::String<64>;
    type MyCounter = u32;
    #[derive(serde::Serialize, serde::Deserialize, Copy, Clone)]
    struct MyComplex {
        re: i8,
        i: i8,
    }

    let my_config_startup: Option<MyConfig> = storage::get("my_config").await.unwrap();
    let my_counter_startup: Option<MyCounter> = storage::get("my_counter").await.unwrap();
    let my_counter_new = my_counter_startup.unwrap_or_default().saturating_add(1);
    storage::insert("my_counter", my_counter_new).await.unwrap();
    info!(
        "Startup value of my_config is {:?}; my_counter was incremented to {}",
        my_config_startup.as_ref(),
        my_counter_new,
    );
    if storage::get::<MyComplex>("my_complex")
        .await
        .unwrap()
        .is_none()
    {
        storage::insert("my_complex", MyComplex { re: 4, i: -3 })
            .await
            .unwrap();
    }

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

    let handler = coap_message_demos::full_application_tree(log)
        .at(
            &["stdout"],
            coap_scroll_ring_server::BufferHandler::new(&buffer),
        )
        .at_with_attributes(
            &["config"],
            &[],
            coap_handler_implementations::TypeHandler::new(CborStorageAccess::<MyConfig>::new(
                "my_config",
            )),
        )
        .at_with_attributes(
            &["counter"],
            &[],
            coap_handler_implementations::TypeHandler::new(CborStorageAccess::<MyCounter>::new(
                "my_counter",
            )),
        )
        .at_with_attributes(
            &["complex"],
            &[],
            coap_handler_implementations::TypeHandler::new(CborStorageAccess::<MyComplex>::new(
                "my_complex",
            )),
        );

    // going with an embassy_futures join instead of RIOT-rs's spawn to avoid the need for making
    // stdout static.
    embassy_futures::join::join(
        riot_rs::coap::coap_run(handler),
        run_client_operations(stdout),
    )
    .await;
}

/// In parallel to server operation, this function performs some operations as a client.
///
/// This doubles as an experimentation ground for the client side of embedded_nal_coap and
/// coap-request in general.
async fn run_client_operations(mut stdout: impl core::fmt::Write) {
    let client = riot_rs::coap::coap_client().await;

    // shame
    let addr = "10.42.0.1:1234";
    let demoserver = addr.parse().unwrap();

    use coap_request::Stack;
    writeln!(stdout, "Sending GET to {}...", addr).unwrap();
    let response = client
        .to(demoserver)
        .request(
            coap_request_implementations::Code::get()
                .with_path("/other/separate")
                .processing_response_payload_through(|p| {
                    writeln!(stdout, "Got payload {:?}", p).unwrap();
                }),
        )
        .await;
    writeln!(
        stdout,
        "Response {:?}",
        response.map_err(|_| "TransportError")
    )
    .unwrap();

    let req = coap_request_implementations::Code::post().with_path("/uppercase");

    writeln!(stdout, "Sending POST...").unwrap();
    let mut response = client.to(demoserver);
    let response = response.request(
        req.with_request_payload_slice(b"Set time to 1955-11-05")
            .processing_response_payload_through(|p| {
                writeln!(stdout, "Uppercase is {}", core::str::from_utf8(p).unwrap()).unwrap();
            }),
    );
    let response = response.await;
    writeln!(
        stdout,
        "Response {:?}",
        response.map_err(|_| "TransportError")
    )
    .unwrap();
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
