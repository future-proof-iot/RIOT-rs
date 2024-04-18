//! This module provides an opinionated integration of `embassy`.

#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

pub mod define_peripherals;

cfg_if::cfg_if! {
    if #[cfg(context = "nrf")] {
        #[path = "arch/nrf/mod.rs"]
        pub mod arch;
    } else if #[cfg(context = "rp2040")] {
        #[path = "arch/rp2040/mod.rs"]
        pub mod arch;
    } else if #[cfg(context = "esp")] {
        #[path = "arch/esp.rs"]
        pub mod arch;
    } else if #[cfg(context = "riot-rs")] {
        compile_error!("this architecture is not supported");
    } else {
        #[path = "arch/dummy/mod.rs"]
        pub mod arch;
    }
}

#[cfg(feature = "usb")]
pub mod usb;

#[cfg(feature = "net")]
pub mod network;

#[cfg(feature = "wifi")]
mod wifi;

use riot_rs_debug::println;

// re-exports
pub use linkme::{self, distributed_slice};
pub use static_cell::make_static;

// Used by a macro we provide
pub use embassy_executor;
pub use embassy_executor::Spawner;

// Crates used in driver configuration functions
#[cfg(feature = "net")]
pub use embassy_net;
#[cfg(feature = "usb")]
pub use embassy_usb;

#[cfg(feature = "usb-ethernet")]
use usb::ethernet::NetworkDevice;

#[cfg(feature = "wifi")]
use wifi::NetworkDevice;

#[cfg(feature = "net")]
pub use network::NetworkStack;

#[cfg(feature = "threading")]
pub mod blocker;
pub mod delegate;
pub mod sendcell;

pub type Task = fn(Spawner, &mut arch::OptionalPeripherals);

#[distributed_slice]
pub static EMBASSY_TASKS: [Task] = [..];

#[cfg(feature = "executor-interrupt")]
pub static EXECUTOR: arch::Executor = arch::Executor::new();

#[cfg(feature = "executor-interrupt")]
#[distributed_slice(riot_rs_rt::INIT_FUNCS)]
pub(crate) fn init() {
    println!("riot-rs-embassy::init()");
    let p = arch::init(Default::default());

    #[cfg(any(context = "nrf", context = "rp2040"))]
    {
        EXECUTOR.start(arch::SWI);
        EXECUTOR.spawner().must_spawn(init_task(p));
    }

    #[cfg(context = "esp")]
    EXECUTOR.run(|spawner| spawner.must_spawn(init_task(p)));
}

#[cfg(feature = "executor-single-thread")]
#[export_name = "riot_rs_embassy_init"]
fn init() -> ! {
    println!("riot-rs-embassy::init()");
    let p = arch::init(Default::default());

    println!("riot-rs-embassy::init() done");

    let executor = make_static!(arch::Executor::new());
    executor.run(|spawner| spawner.must_spawn(init_task(p)));
}

#[embassy_executor::task]
async fn init_task(mut peripherals: arch::OptionalPeripherals) {
    println!("riot-rs-embassy::init_task()");

    #[cfg(feature = "hwrng")]
    arch::hwrng::construct_rng(&mut peripherals);
    // Clock startup and entropy collection may lend themselves to parallelization, provided that
    // doesn't impact runtime RAM or flash use.

    #[cfg(all(context = "nrf", feature = "usb"))]
    {
        // nrf52840
        let clock: embassy_nrf::pac::CLOCK = unsafe { core::mem::transmute(()) };

        println!("nrf: enabling ext hfosc...");
        clock.tasks_hfclkstart.write(|w| unsafe { w.bits(1) });
        while clock.events_hfclkstarted.read().bits() != 1 {}
    }

    let spawner = Spawner::for_current_executor().await;

    for task in EMBASSY_TASKS {
        task(spawner, &mut peripherals);
    }

    #[cfg(feature = "usb")]
    let mut usb_builder = {
        let usb_config = usb::config();

        let usb_driver = arch::usb::driver(&mut peripherals);

        // Create embassy-usb DeviceBuilder using the driver and config.
        let builder = usb::UsbBuilder::new(
            usb_driver,
            usb_config,
            &mut make_static!([0; 256])[..],
            &mut make_static!([0; 256])[..],
            &mut make_static!([0; 256])[..],
            &mut make_static!([0; 128])[..],
            &mut make_static!([0; 128])[..],
        );

        builder
    };

    #[cfg(feature = "usb-ethernet")]
    let device = {
        use embassy_usb::class::cdc_ncm::{
            embassy_net::State as NetState, CdcNcmClass, State as CdcNcmState,
        };

        // Host's MAC addr. This is the MAC the host "thinks" its USB-to-ethernet adapter has.
        let host_mac_addr = [0x8A, 0x88, 0x88, 0x88, 0x88, 0x88];

        // Create classes on the builder.
        let usb_cdc_ecm = CdcNcmClass::new(
            &mut usb_builder,
            make_static!(CdcNcmState::new()),
            host_mac_addr,
            64,
        );

        let our_mac_addr = [0xCA, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC];

        let (runner, device) = usb_cdc_ecm
            .into_embassy_net_device::<{ network::ETHERNET_MTU }, 4, 4>(
                make_static!(NetState::new()),
                our_mac_addr,
            );

        spawner.spawn(usb::ethernet::usb_ncm_task(runner)).unwrap();

        device
    };

    #[cfg(feature = "usb")]
    {
        for hook in usb::USB_BUILDER_HOOKS {
            hook.lend(&mut usb_builder).await;
        }
        let usb = usb_builder.build();
        spawner.spawn(usb::usb_task(usb)).unwrap();
    }

    #[cfg(feature = "wifi-cyw43")]
    let (device, control) = {
        let (net_device, control) = wifi::cyw43::device(&mut peripherals, &spawner).await;
        (net_device, control)
    };

    #[cfg(feature = "wifi-esp")]
    let device = wifi::esp_wifi::init(&mut peripherals, spawner);

    #[cfg(feature = "net")]
    {
        use crate::network::STACK;
        use crate::sendcell::SendCell;
        use embassy_net::{Stack, StackResources};

        const MAX_CONCURRENT_SOCKETS: usize = riot_rs_utils::usize_from_env_or!(
            "CONFIG_NETWORK_MAX_CONCURRENT_SOCKETS",
            4,
            "maximum number of concurrent sockets allowed by the network stack"
        );

        let config = network::config();

        // Generate random seed
        // let mut rng = Rng::new(p.RNG, Irqs);
        // let mut seed = [0; 8];
        // rng.blocking_fill_bytes(&mut seed);
        // let seed = u64::from_le_bytes(seed);
        let seed = 1234u64;

        // Init network stack
        let stack = &*make_static!(Stack::new(
            device,
            config,
            make_static!(StackResources::<MAX_CONCURRENT_SOCKETS>::new()),
            seed
        ));

        spawner.spawn(network::net_task(stack)).unwrap();

        if STACK
            .lock(|c| c.set(SendCell::new(stack, spawner)))
            .is_err()
        {
            unreachable!();
        }
    }

    #[cfg(feature = "wifi-cyw43")]
    {
        wifi::cyw43::join(control).await;
    };

    // mark used
    let _ = peripherals;

    println!("riot-rs-embassy::init_task() done");
}
