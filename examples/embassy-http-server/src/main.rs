#![no_main]
#![no_std]
#![feature(impl_trait_in_assoc_type)]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

mod pins;
mod routes;

use riot_rs::{debug::log::*, network, ConstStaticCell, Spawner, StaticCell};

use embassy_net::tcp::TcpSocket;
use embassy_time::Duration;
use picoserve::io::Error;

#[cfg(feature = "button-readings")]
use embassy_nrf::gpio::{Input, Pin, Pull};

#[cfg(feature = "button-readings")]
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};

#[cfg(feature = "button-readings")]
#[derive(Copy, Clone)]
struct ButtonInputs(&'static Mutex<CriticalSectionRawMutex, Buttons>);

#[cfg(feature = "button-readings")]
struct Buttons {
    button1: Input<'static>,
    button2: Input<'static>,
    button3: Input<'static>,
    button4: Input<'static>,
}

#[cfg(feature = "button-readings")]
impl picoserve::extract::FromRef<AppState> for ButtonInputs {
    fn from_ref(state: &AppState) -> Self {
        state.buttons
    }
}

mod approuter {
    use picoserve::routing::get;

    use crate::routes;

    pub struct AppState {
        #[cfg(feature = "button-readings")]
        pub buttons: crate::ButtonInputs,
    }

    pub type AppRouter = impl picoserve::routing::PathRouter<AppState>;

    pub fn make_app() -> picoserve::Router<AppRouter, AppState> {
        let router = picoserve::Router::new().route("/", get(routes::index));
        #[cfg(feature = "button-readings")]
        let router = router.route("/buttons", get(routes::buttons));
        router
    }
}

use approuter::*;

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

        info!("{}: Listening on TCP:80...", id);
        if let Err(e) = socket.accept(80).await {
            info!("{}: accept error: {:?}", id, e);
            continue;
        }

        let remote_endpoint = socket.remote_endpoint();

        info!("{}: Received connection from {:?}", id, remote_endpoint);

        match picoserve::serve_with_state(app, config, &mut [0; 2048], socket, &state).await {
            Ok(handled_requests_count) => {
                info!(
                    "{} requests handled from {:?}",
                    handled_requests_count, remote_endpoint,
                );
            }
            Err(err) => info!("{:?}", err.kind()),
        }
    }
}

#[riot_rs::spawner(autostart, peripherals)]
fn main(spawner: Spawner, peripherals: pins::Peripherals) {
    #[cfg(not(feature = "button-readings"))]
    let _ = peripherals;

    #[cfg(feature = "button-readings")]
    let button_inputs = {
        let buttons = peripherals.buttons;

        let buttons = Buttons {
            button1: Input::new(buttons.btn1.degrade(), Pull::Up),
            button2: Input::new(buttons.btn2.degrade(), Pull::Up),
            button3: Input::new(buttons.btn3.degrade(), Pull::Up),
            button4: Input::new(buttons.btn4.degrade(), Pull::Up),
        };

        static BUTTON_INPUTS: StaticCell<Mutex<CriticalSectionRawMutex, Buttons>> =
            StaticCell::new();
        ButtonInputs(BUTTON_INPUTS.init_with(|| Mutex::new(buttons)))
    };

    static APP: StaticCell<picoserve::Router<AppRouter, AppState>> = StaticCell::new();
    let app = APP.init_with(|| make_app());

    static CONFIG: ConstStaticCell<picoserve::Config<Duration>> =
        ConstStaticCell::new(picoserve::Config::new(picoserve::Timeouts {
            start_read_request: Some(Duration::from_secs(5)),
            read_request: Some(Duration::from_secs(1)),
            write: Some(Duration::from_secs(1)),
        }));

    let config = CONFIG.take();

    for id in 0..WEB_TASK_POOL_SIZE {
        let app_state = AppState {
            #[cfg(feature = "button-readings")]
            buttons: button_inputs,
        };
        spawner.spawn(web_task(id, app, config, app_state)).unwrap();
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
fn usb_config() -> riot_rs::embassy_usb::Config<'static> {
    let mut config = riot_rs::embassy_usb::Config::new(0xc0de, 0xcafe);
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
