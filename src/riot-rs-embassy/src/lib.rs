//! This module provides an opinionated integration of `embassy`.

#![no_std]
#![feature(impl_trait_in_assoc_type)]
#![feature(used_with_arg)]
#![feature(doc_auto_cfg)]

pub mod define_peripherals;
pub mod gpio;

cfg_if::cfg_if! {
    if #[cfg(context = "nrf")] {
        pub use riot_rs_nrf as arch;
    } else if #[cfg(context = "rp")] {
        pub use riot_rs_rp as arch;
    } else if #[cfg(context = "esp")] {
        pub use riot_rs_esp as arch;
    } else if #[cfg(context = "stm32")] {
        pub use riot_rs_stm32 as arch;
    } else if #[cfg(context = "riot-rs")] {
        compile_error!("this architecture is not supported");
    } else {
        pub mod arch;
    }
}

#[cfg(feature = "i2c")]
pub mod i2c;

#[cfg(feature = "spi")]
pub mod spi;

#[cfg(feature = "usb")]
pub mod usb;

#[cfg(feature = "net")]
pub mod network;

#[cfg(feature = "wifi")]
mod wifi;

use riot_rs_debug::log::debug;

// re-exports
pub use linkme::{self, distributed_slice};
pub use static_cell::{ConstStaticCell, StaticCell};

// All items of this module are re-exported at the root of `riot_rs`.
pub mod api {
    #[cfg(feature = "i2c")]
    pub use crate::i2c;

    #[cfg(feature = "spi")]
    pub use crate::spi;

    #[cfg(feature = "threading")]
    pub use crate::blocker;
    #[cfg(feature = "usb")]
    pub use crate::usb;
    pub use crate::{
        arch, define_peripherals, delegate, gpio, group_peripherals, Spawner, EMBASSY_TASKS,
    };
    #[cfg(feature = "net")]
    pub use crate::{network, NetworkStack};

    #[cfg(feature = "executor-interrupt")]
    pub use crate::arch::EXECUTOR;

    #[cfg(feature = "executor-thread")]
    pub use crate::thread_executor;
}

// These are made available in `riot_rs::reexports`.
pub mod reexports {
    #[cfg(feature = "net")]
    pub use embassy_net;
    #[cfg(feature = "usb")]
    pub use embassy_usb;
    // Used by a macro we provide
    pub use embassy_executor;
}

pub use embassy_executor::Spawner;

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

#[cfg(feature = "executor-thread")]
pub mod thread_executor;

pub type Task = fn(Spawner, &mut arch::OptionalPeripherals);

#[distributed_slice]
pub static EMBASSY_TASKS: [Task] = [..];

#[cfg(not(any(
    feature = "executor-interrupt",
    feature = "executor-none",
    feature = "executor-single-thread",
    feature = "executor-thread"
)))]
compile_error!(
    r#"must select one of "executor-interrupt", "executor-single-thread", "executor-thread", "executor-none"!"#
);

#[cfg(all(feature = "threading", feature = "executor-single-thread"))]
compile_error!(r#""executor-single-thread" and "threading" are mutually exclusive!"#);

#[cfg(feature = "executor-interrupt")]
#[distributed_slice(riot_rs_rt::INIT_FUNCS)]
pub(crate) fn init() {
    debug!("riot-rs-embassy::init(): using interrupt mode executor");
    let p = arch::init();

    #[cfg(any(context = "nrf", context = "rp2040", context = "stm32"))]
    {
        arch::EXECUTOR.start(arch::SWI);
        arch::EXECUTOR.spawner().must_spawn(init_task(p));
    }

    #[cfg(context = "esp")]
    EXECUTOR.run(|spawner| spawner.must_spawn(init_task(p)));
}

#[cfg(feature = "executor-single-thread")]
#[export_name = "riot_rs_embassy_init"]
fn init() -> ! {
    debug!("riot-rs-embassy::init(): using single thread executor");
    let p = arch::init();

    static EXECUTOR: StaticCell<arch::Executor> = StaticCell::new();
    EXECUTOR
        .init_with(|| arch::Executor::new())
        .run(|spawner| spawner.must_spawn(init_task(p)))
}

#[cfg(feature = "executor-thread")]
mod executor_thread {
    pub(crate) const STACKSIZE: usize = riot_rs_utils::usize_from_env_or!(
        "CONFIG_EXECUTOR_THREAD_STACKSIZE",
        16384,
        "executor thread stack size"
    );

    pub(crate) const PRIORITY: u8 = riot_rs_utils::u8_from_env_or!(
        "CONFIG_EXECUTOR_THREAD_PRIORITY",
        8,
        "executor thread priority"
    );
}

#[cfg(feature = "executor-thread")]
#[riot_rs_macros::thread(autostart, stacksize = executor_thread::STACKSIZE, priority = executor_thread::PRIORITY)]
fn init() {
    debug!("riot-rs-embassy::init(): using thread executor");
    let p = arch::init();

    static EXECUTOR: StaticCell<thread_executor::Executor> = StaticCell::new();
    EXECUTOR
        .init_with(|| thread_executor::Executor::new())
        .run(|spawner| spawner.must_spawn(init_task(p)));
}

#[embassy_executor::task]
async fn init_task(mut peripherals: arch::OptionalPeripherals) {
    debug!("riot-rs-embassy::init_task()");

    #[cfg(all(context = "stm32", feature = "external-interrupts"))]
    arch::extint_registry::EXTINT_REGISTRY.init(&mut peripherals);

    #[cfg(context = "esp")]
    arch::gpio::init(&mut peripherals);

    #[cfg(feature = "i2c")]
    arch::i2c::init(&mut peripherals);

    #[cfg(feature = "spi")]
    arch::spi::init(&mut peripherals);

    #[cfg(feature = "hwrng")]
    arch::hwrng::construct_rng(&mut peripherals);
    // Clock startup and entropy collection may lend themselves to parallelization, provided that
    // doesn't impact runtime RAM or flash use.

    #[cfg(all(feature = "usb", context = "nrf"))]
    arch::usb::init();

    let spawner = Spawner::for_current_executor().await;

    for task in EMBASSY_TASKS {
        task(spawner, &mut peripherals);
    }

    #[cfg(feature = "usb")]
    let mut usb_builder = {
        let usb_config = usb::config();

        let usb_driver = arch::usb::driver(&mut peripherals);

        static CONFIG_DESC: ConstStaticCell<[u8; 256]> = ConstStaticCell::new([0; 256]);
        static BOS_DESC: ConstStaticCell<[u8; 256]> = ConstStaticCell::new([0; 256]);
        static MSOS_DESC: ConstStaticCell<[u8; 128]> = ConstStaticCell::new([0; 128]);
        static CONTROL_BUF: ConstStaticCell<[u8; 128]> = ConstStaticCell::new([0; 128]);
        // Create embassy-usb DeviceBuilder using the driver and config.
        let builder = usb::UsbBuilder::new(
            usb_driver,
            usb_config,
            CONFIG_DESC.take(),
            BOS_DESC.take(),
            MSOS_DESC.take(),
            CONTROL_BUF.take(),
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
        static CDC_ECM_STATE: StaticCell<CdcNcmState> = StaticCell::new();
        let usb_cdc_ecm = CdcNcmClass::new(
            &mut usb_builder,
            CDC_ECM_STATE.init_with(|| CdcNcmState::new()),
            host_mac_addr,
            64,
        );

        let our_mac_addr = [0xCA, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC];

        static NET_STATE: StaticCell<NetState<{ network::ETHERNET_MTU }, 4, 4>> = StaticCell::new();
        let (runner, device) = usb_cdc_ecm
            .into_embassy_net_device::<{ network::ETHERNET_MTU }, 4, 4>(
                NET_STATE.init_with(|| NetState::new()),
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
        let (net_device, control) = riot_rs_rp::cyw43::device(&mut peripherals, &spawner).await;
        (net_device, control)
    };

    #[cfg(feature = "wifi-esp")]
    let device = arch::wifi::esp_wifi::init(&mut peripherals, spawner);

    #[cfg(feature = "net")]
    {
        use embassy_net::StackResources;

        use crate::sendcell::SendCell;

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
        static RESOURCES: StaticCell<StackResources<MAX_CONCURRENT_SOCKETS>> = StaticCell::new();
        let (stack, runner) = embassy_net::new(
            device,
            config,
            RESOURCES.init_with(|| StackResources::new()),
            seed,
        );

        spawner.spawn(network::net_task(runner)).unwrap();

        if crate::network::STACK
            .init(SendCell::new(stack, spawner))
            .is_err()
        {
            unreachable!();
        }
    }

    #[cfg(feature = "wifi-cyw43")]
    {
        riot_rs_rp::cyw43::join(control).await;
    };

    // mark used
    let _ = peripherals;

    debug!("riot-rs-embassy::init_task() done");
}
