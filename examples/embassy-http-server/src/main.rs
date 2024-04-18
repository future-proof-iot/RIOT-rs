#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]
// Both of these are required for sensors
#![feature(trait_upcasting)]
#![feature(impl_trait_in_assoc_type)]

mod pins;
mod routes;
mod sensors;

use riot_rs::{
    debug::println,
    embassy::{network, Spawner},
    sensors::Sensor,
};

use embassy_net::tcp::TcpSocket;
use embassy_time::Duration;
use picoserve::routing::get;
use static_cell::make_static;

struct AppState {}

type AppRouter = impl picoserve::routing::PathRouter<AppState>;

const WEB_TASK_POOL_SIZE: usize = 2;

#[riot_rs::task(pool_size = WEB_TASK_POOL_SIZE)]
async fn web_task(
    id: usize,
    app: &'static picoserve::Router<AppRouter, AppState>,
    config: &'static picoserve::Config<Duration>,
    state: AppState,
) -> ! {
    let stack = network::network_stack().await.unwrap();

    let mut rx_buffer = [0; 1024];
    let mut tx_buffer = [0; 1024];

    loop {
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);

        println!("{}: Listening on TCP:80...", id);
        if let Err(e) = socket.accept(80).await {
            println!("{}: accept error: {:?}", id, e);
            continue;
        }

        let remote_endpoint = socket.remote_endpoint();

        println!("{}: Received connection from {:?}", id, remote_endpoint);

        match picoserve::serve_with_state(app, config, &mut [0; 2048], socket, &state).await {
            Ok(handled_requests_count) => {
                println!(
                    "{} requests handled from {:?}",
                    handled_requests_count, remote_endpoint,
                );
            }
            Err(err) => println!("{:?}", err),
        }
    }
}

#[riot_rs::spawner(autostart, peripherals)]
fn main(spawner: Spawner, _peripherals: pins::Peripherals) {
    #[cfg(context = "nrf52")]
    {
        use riot_rs::sensors::sensor::{PhysicalValue, ThresholdKind};
        let threshold = PhysicalValue::new(2300);
        sensors::TEMP_SENSOR.set_threshold(ThresholdKind::Lower, threshold);
        sensors::TEMP_SENSOR.set_threshold_enabled(ThresholdKind::Lower, true);
    }

    fn make_app() -> picoserve::Router<AppRouter, AppState> {
        let router = picoserve::Router::new().route("/", get(routes::index));
        #[cfg(feature = "button-readings")]
        let router = router.route("/api/buttons", get(routes::buttons));
        let router = router.route("/api/sensors", get(routes::sensors));
        #[cfg(context = "nrf52840")]
        let router = router.route("/api/temp", get(routes::temp));
        router
    }

    let app = make_static!(make_app());

    let config = make_static!(picoserve::Config::new(picoserve::Timeouts {
        start_read_request: Some(Duration::from_secs(5)),
        read_request: Some(Duration::from_secs(1)),
        write: Some(Duration::from_secs(1)),
    }));

    for id in 0..WEB_TASK_POOL_SIZE {
        let app_state = AppState {};
        spawner.spawn(web_task(id, app, config, app_state)).unwrap();
    }
}

#[riot_rs::task(autostart)]
async fn temp_subscriber() {
    let rx = sensors::TEMP_SENSOR.subscribe();

    loop {
        let notification = rx.receive().await;
        println!("{:#?}", notification);
    }
}

#[riot_rs::config(network)]
fn network_config() -> embassy_net::Config {
    use embassy_net::Ipv4Address;

    embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
        address: embassy_net::Ipv4Cidr::new(Ipv4Address::new(10, 42, 0, 61), 24),
        dns_servers: heapless::Vec::new(),
        gateway: Some(Ipv4Address::new(10, 42, 0, 1)),
    })
}

#[cfg(capability = "hw/usb-device-port")]
#[riot_rs::config(usb)]
fn usb_config() -> riot_rs::embassy::embassy_usb::Config<'static> {
    let mut config = riot_rs::embassy::embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("HTTP-over-USB-Ethernet example");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    // Required for Windows support.
    config.composite_with_iads = true;
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config
}
