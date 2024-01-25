#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

mod pins;
mod routes;

use riot_rs as _;

use riot_rs::embassy::{
    arch::OptionalPeripherals, Application, ApplicationInitError, Drivers, InitializationArgs,
    NetworkStack,
};
use riot_rs::rt::debug::println;

use embassy_net::tcp::TcpSocket;
use embassy_nrf::gpio::{Input, Pin, Pull};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embassy_time::Duration;
use picoserve::routing::get;
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

struct AppState {
    buttons: ButtonInputs,
}

#[derive(Copy, Clone)]
struct ButtonInputs(&'static Mutex<CriticalSectionRawMutex, Buttons>);

struct Buttons {
    button1: Input<'static>,
    button2: Input<'static>,
    button3: Input<'static>,
    button4: Input<'static>,
}

impl picoserve::extract::FromRef<AppState> for ButtonInputs {
    fn from_ref(state: &AppState) -> Self {
        state.buttons
    }
}

type AppRouter = impl picoserve::routing::PathRouter<AppState>;

const WEB_TASK_POOL_SIZE: usize = 2;

#[embassy_executor::task(pool_size = WEB_TASK_POOL_SIZE)]
async fn web_task(
    id: usize,
    stack: &'static NetworkStack,
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

struct WebServer {
    button_inputs: ButtonInputs,
}

impl Application for WebServer {
    fn initialize(
        peripherals: &mut OptionalPeripherals,
        _init_args: InitializationArgs,
    ) -> Result<&dyn Application, ApplicationInitError> {
        let buttons = pins::Buttons::take_from(peripherals)?;

        let buttons = Buttons {
            button1: Input::new(buttons.btn1.degrade(), Pull::Up),
            button2: Input::new(buttons.btn2.degrade(), Pull::Up),
            button3: Input::new(buttons.btn3.degrade(), Pull::Up),
            button4: Input::new(buttons.btn4.degrade(), Pull::Up),
        };

        let button_inputs = ButtonInputs(make_static!(Mutex::new(buttons)));

        Ok(make_static!(Self { button_inputs }))
    }

    fn start(&self, spawner: embassy_executor::Spawner, drivers: Drivers) {
        let stack = drivers.stack.get().unwrap();

        fn make_app() -> picoserve::Router<AppRouter, AppState> {
            picoserve::Router::new()
                .route("/", get(routes::index))
                .route("/buttons", get(routes::buttons))
        }

        let app = make_static!(make_app());

        let config = make_static!(picoserve::Config {
            start_read_request_timeout: Some(Duration::from_secs(5)),
            read_request_timeout: Some(Duration::from_secs(1)),
            write_timeout: Some(Duration::from_secs(1)),
        });

        for id in 0..WEB_TASK_POOL_SIZE {
            let app_state = AppState {
                buttons: self.button_inputs,
            };
            spawner
                .spawn(web_task(id, stack, app, config, app_state))
                .unwrap();
        }
    }
}

riot_rs::embassy::riot_initialize!(WebServer);
