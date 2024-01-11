#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use riot_rs as _;

use riot_rs::embassy::{
    arch, Application, ApplicationInitError, Drivers, InitializationArgs, UsbEthernetStack,
};
use riot_rs::rt::debug::println;

use embassy_net::tcp::TcpSocket;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embassy_time::Duration;
use picoserve::{
    response::{DebugValue, IntoResponse},
    routing::{get, parse_path_segment},
};
use static_cell::make_static;

struct EmbassyTimer;

impl picoserve::Timer for EmbassyTimer {
    type Duration = embassy_time::Duration;
    type TimeoutError = embassy_time::TimeoutError;

    async fn run_with_timeout<F: core::future::Future>(
        &mut self,
        duration: Self::Duration,
        future: F,
    ) -> Result<F::Output, Self::TimeoutError> {
        embassy_time::with_timeout(duration, future).await
    }
}

struct AppState {}

type AppRouter = impl picoserve::routing::PathRouter<AppState>;

const WEB_TASK_POOL_SIZE: usize = 2;

#[embassy_executor::task(pool_size = WEB_TASK_POOL_SIZE)]
async fn web_task(
    id: usize,
    stack: &'static UsbEthernetStack,
    app: &'static picoserve::Router<AppRouter, AppState>,
    config: &'static picoserve::Config<Duration>,
    state: AppState,
) -> ! {
    let mut rx_buffer = [0; 1024];
    let mut tx_buffer = [0; 1024];

    loop {
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);

        println!("{}: Listening on TCP:80...", id);
        if let Err(e) = socket.accept(80).await {
            println!("{}: accept error: {:?}", id, e);
            continue;
        }

        println!(
            "{}: Received connection from {:?}",
            id,
            socket.remote_endpoint()
        );

        let (socket_rx, socket_tx) = socket.split();

        match picoserve::serve_with_state(
            app,
            EmbassyTimer,
            config,
            &mut [0; 2048],
            socket_rx,
            socket_tx,
            &state,
        )
        .await
        {
            Ok(handled_requests_count) => {
                println!(
                    "{} requests handled from {:?}",
                    handled_requests_count,
                    socket.remote_endpoint()
                );
            }
            Err(err) => println!("{:?}", err),
        }
    }
}

async fn index() -> impl IntoResponse {
    picoserve::response::File::html(include_str!("../static/index.html"))
}

#[embassy_executor::task]
async fn serve(spawner: embassy_executor::Spawner, stack: &'static UsbEthernetStack) {
    fn make_app() -> picoserve::Router<AppRouter, AppState> {
        picoserve::Router::new().route("/", get(index))
    }

    let app = make_static!(make_app());

    let config = make_static!(picoserve::Config {
        start_read_request_timeout: Some(Duration::from_secs(5)),
        read_request_timeout: Some(Duration::from_secs(1)),
        write_timeout: Some(Duration::from_secs(1)),
    });

    for id in 0..WEB_TASK_POOL_SIZE {
        spawner
            .spawn(web_task(id, &stack, app, config, AppState {}))
            .unwrap();
    }
}

struct WebServer {}

impl Application for WebServer {
    fn initialize(
        _peripherals: &mut arch::OptionalPeripherals,
        _init_args: InitializationArgs,
    ) -> Result<&dyn Application, ApplicationInitError> {
        Ok(&Self {})
    }

    fn start(&self, spawner: embassy_executor::Spawner, drivers: Drivers) {
        let stack = drivers.stack.get().unwrap();
        spawner.spawn(serve(spawner, stack)).unwrap();
    }
}

riot_rs::embassy::riot_initialize!(WebServer);

#[no_mangle]
fn riot_main() {
    println!(
        "Hello from riot_main()! Running on a {} board.",
        riot_rs::buildinfo::BOARD
    );

    loop {}
}
