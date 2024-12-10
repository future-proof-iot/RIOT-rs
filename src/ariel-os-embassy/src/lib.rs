//! This module provides an opinionated integration of `embassy`.

#![no_std]
#![feature(impl_trait_in_assoc_type)]
#![feature(used_with_arg)]
#![feature(doc_auto_cfg)]

pub mod define_peripherals;
pub mod gpio;

pub use ariel_os_hal as hal;

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

use ariel_os_debug::log::debug;

// re-exports
pub use linkme::{self, distributed_slice};
pub use static_cell::{ConstStaticCell, StaticCell};

// All items of this module are re-exported at the root of `ariel_os`.
pub mod api {
    pub use crate::{
        asynch, define_peripherals, delegate, gpio, group_peripherals, hal, EMBASSY_TASKS,
    };

    #[cfg(feature = "time")]
    pub mod time {
        //! Provides time-related facilities.
        // NOTE: we may want to re-export more items in the future, but not re-export the whole
        // crate.
        pub use embassy_time::{Delay, Duration, Instant, Timer, TICK_HZ};
    }

    #[cfg(feature = "i2c")]
    pub use crate::i2c;
    #[cfg(feature = "net")]
    pub use crate::network;
    #[cfg(feature = "spi")]
    pub use crate::spi;
    #[cfg(feature = "usb")]
    pub use crate::usb;
}

// These are made available in `ariel_os::reexports`.
pub mod reexports {
    #[cfg(feature = "net")]
    pub use embassy_net;
    #[cfg(feature = "time")]
    pub use embassy_time;
    #[cfg(feature = "usb")]
    pub use embassy_usb;
    #[cfg(feature = "usb-hid")]
    pub use usbd_hid;
    // Used by a macro we provide
    pub use embassy_executor;
}

#[cfg(feature = "net")]
cfg_if::cfg_if! {
    if #[cfg(feature = "usb-ethernet")] {
        use usb::ethernet::NetworkDevice;
    } else if #[cfg(feature = "wifi")] {
        use wifi::NetworkDevice;
    } else if #[cfg(context = "ariel-os")] {
        compile_error!("no backend for net is active");
    } else {
        use network::DummyDriver as NetworkDevice;
    }
}

#[cfg(feature = "net")]
pub use network::NetworkStack;

pub mod asynch;
pub mod delegate;
pub mod sendcell;

#[cfg(feature = "executor-thread")]
pub mod thread_executor;

pub type Task = fn(asynch::Spawner, &mut hal::OptionalPeripherals);

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
#[distributed_slice(ariel_os_rt::INIT_FUNCS)]
pub(crate) fn init() {
    debug!("ariel-os-embassy::init(): using interrupt mode executor");
    let p = hal::init();

    #[cfg(any(context = "nrf", context = "rp2040", context = "stm32"))]
    {
        hal::EXECUTOR.start(hal::SWI);
        hal::EXECUTOR.spawner().must_spawn(init_task(p));
    }

    #[cfg(context = "esp")]
    EXECUTOR.run(|spawner| spawner.must_spawn(init_task(p)));
}

#[cfg(feature = "executor-single-thread")]
#[export_name = "__ariel_os_embassy_init"]
fn init() -> ! {
    debug!("ariel-os-embassy::init(): using single thread executor");
    let p = hal::init();

    static EXECUTOR: StaticCell<hal::Executor> = StaticCell::new();
    EXECUTOR
        .init_with(|| hal::Executor::new())
        .run(|spawner| spawner.must_spawn(init_task(p)))
}

#[cfg(feature = "executor-thread")]
mod executor_thread {
    pub(crate) const STACKSIZE: usize = ariel_os_utils::usize_from_env_or!(
        "CONFIG_EXECUTOR_THREAD_STACKSIZE",
        16384,
        "executor thread stack size"
    );

    pub(crate) const PRIORITY: u8 = ariel_os_utils::u8_from_env_or!(
        "CONFIG_EXECUTOR_THREAD_PRIORITY",
        8,
        "executor thread priority"
    );
}

#[cfg(feature = "executor-thread")]
#[ariel_os_macros::thread(autostart, no_wait, stacksize = executor_thread::STACKSIZE, priority = executor_thread::PRIORITY)]
fn init() {
    debug!("ariel-os-embassy::init(): using thread executor");
    let p = hal::init();

    static EXECUTOR: StaticCell<thread_executor::Executor> = StaticCell::new();
    EXECUTOR
        .init_with(|| thread_executor::Executor::new())
        .run(|spawner| spawner.must_spawn(init_task(p)));
}

#[embassy_executor::task]
async fn init_task(mut peripherals: hal::OptionalPeripherals) {
    let spawner = asynch::Spawner::for_current_executor().await;
    asynch::set_spawner(spawner.make_send());

    debug!("ariel-os-embassy::init_task()");

    #[cfg(all(context = "stm32", feature = "external-interrupts"))]
    hal::extint_registry::EXTINT_REGISTRY.init(&mut peripherals);

    #[cfg(context = "esp")]
    hal::gpio::init(&mut peripherals);

    #[cfg(feature = "i2c")]
    hal::i2c::init(&mut peripherals);

    #[cfg(feature = "spi")]
    hal::spi::init(&mut peripherals);

    #[cfg(feature = "hwrng")]
    hal::hwrng::construct_rng(&mut peripherals);
    // Clock startup and entropy collection may lend themselves to parallelization, provided that
    // doesn't impact runtime RAM or flash use.

    #[cfg(feature = "storage")]
    ariel_os_storage::init(&mut peripherals).await;

    #[cfg(all(feature = "usb", context = "nrf"))]
    hal::usb::init();

    // Move out the peripherals required for drivers, so that tasks cannot mistakenly take them.
    #[cfg(feature = "usb")]
    let usb_peripherals = hal::usb::Peripherals::new(&mut peripherals);

    // Tasks have to be started before driver initializations so that the tasks are able to
    // configure the drivers using hooks.
    for task in EMBASSY_TASKS {
        task(spawner, &mut peripherals);
    }

    #[cfg(feature = "usb")]
    let mut usb_builder = {
        let usb_config = usb::config();

        let usb_driver = hal::usb::driver(usb_peripherals);

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
        use ariel_os_embassy_common::identity::DeviceId;
        use embassy_usb::class::cdc_ncm::{
            embassy_net::State as NetState, CdcNcmClass, State as CdcNcmState,
        };

        // Host's MAC addr. This is the MAC the host "thinks" its USB-to-ethernet adapter has.
        let host_mac_addr = crate::hal::identity::DeviceId::get()
            .map(|d| d.interface_eui48(1))
            .unwrap_or([0x8A, 0x88, 0x88, 0x88, 0x88, 0x88]);

        // Create classes on the builder.
        static CDC_ECM_STATE: StaticCell<CdcNcmState> = StaticCell::new();
        let usb_cdc_ecm = CdcNcmClass::new(
            &mut usb_builder,
            CDC_ECM_STATE.init_with(CdcNcmState::new),
            host_mac_addr,
            64,
        );

        let our_mac_addr = crate::hal::identity::DeviceId::get()
            .map(|d| d.interface_eui48(0))
            .unwrap_or([0xCA, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC]);

        static NET_STATE: StaticCell<NetState<{ network::ETHERNET_MTU }, 4, 4>> = StaticCell::new();
        let (runner, device) = usb_cdc_ecm
            .into_embassy_net_device::<{ network::ETHERNET_MTU }, 4, 4>(
                NET_STATE.init_with(NetState::new),
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
        let (net_device, control) = hal::cyw43::device(&mut peripherals, &spawner).await;
        (net_device, control)
    };

    #[cfg(feature = "wifi-esp")]
    let device = hal::wifi::esp_wifi::init(&mut peripherals, spawner);

    #[cfg(feature = "net")]
    {
        use embassy_net::StackResources;

        use crate::sendcell::SendCell;

        const MAX_CONCURRENT_SOCKETS: usize = ariel_os_utils::usize_from_env_or!(
            "CONFIG_NETWORK_MAX_CONCURRENT_SOCKETS",
            4,
            "maximum number of concurrent sockets allowed by the network stack"
        );

        #[cfg(not(any(feature = "usb-ethernet", feature = "wifi-cyw43", feature = "wifi-esp")))]
        // The creation of `device` is not organized in such a way that they could be put in a
        // cfg-if without larger refactoring; relying on unused variable lints to keep the
        // condition list up to date.
        let device: NetworkDevice = network::new_dummy();

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
            RESOURCES.init_with(StackResources::new),
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
        hal::cyw43::join(control).await;
    };

    // mark used
    let _ = peripherals;

    debug!("ariel-os-embassy::init_task() done");

    #[cfg(feature = "threading")]
    ariel_os_threads::events::THREAD_START_EVENT.set();
}
