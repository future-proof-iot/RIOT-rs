#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use embassy_executor::{InterruptExecutor, Spawner};
use embassy_usb::{Builder, UsbDevice};
use static_cell::make_static;

pub mod blocker;

pub static EXECUTOR: InterruptExecutor = InterruptExecutor::new();

#[cfg(context = "nrf52")]
mod nrf52 {
    pub use embassy_nrf::interrupt;
    pub use embassy_nrf::interrupt::SWI0_EGU0 as SWI;
    pub use embassy_nrf::{init, Peripherals};

    use embassy_nrf::{bind_interrupts, peripherals, rng, usb};

    bind_interrupts!(struct Irqs {
        USBD => usb::InterruptHandler<peripherals::USBD>;
        POWER_CLOCK => usb::vbus_detect::InterruptHandler;
        RNG => rng::InterruptHandler<peripherals::RNG>;
    });

    #[interrupt]
    unsafe fn SWI0_EGU0() {
        crate::EXECUTOR.on_interrupt()
    }

    // nrf52 usb begin
    use embassy_nrf::usb::{vbus_detect::HardwareVbusDetect, Driver};
    pub type UsbDriver = Driver<'static, peripherals::USBD, HardwareVbusDetect>;
    pub fn usb_driver(usbd: peripherals::USBD) -> UsbDriver {
        Driver::new(usbd, Irqs, HardwareVbusDetect::new(Irqs))
    }
}

#[cfg(context = "rp2040")]
mod rp2040 {
    pub use embassy_rp::interrupt;
    pub use embassy_rp::interrupt::SWI_IRQ_1 as SWI;
    pub use embassy_rp::{init, Peripherals};

    use embassy_rp::{
        bind_interrupts, peripherals,
        peripherals::USB,
        usb::{Driver, InterruptHandler},
    };

    // rp2040 usb start
    bind_interrupts!(struct Irqs {
        USBCTRL_IRQ => InterruptHandler<USB>;
    });

    pub type UsbDriver = Driver<'static, peripherals::USB>;
    #[interrupt]
    unsafe fn SWI_IRQ_1() {
        crate::EXECUTOR.on_interrupt()
    }
    pub fn usb_driver(usb: peripherals::USB) -> UsbDriver {
        Driver::new(usb, Irqs)
    }
}

#[cfg(context = "nrf52")]
use nrf52 as arch;

#[cfg(context = "rp2040")]
use rp2040 as arch;

use arch::SWI;

//
// usb common start
use arch::UsbDriver;
#[embassy_executor::task]
async fn usb_task(mut device: UsbDevice<'static, UsbDriver>) -> ! {
    device.run().await
}
// usb common end
//

//
// usb net begin
const MTU: usize = 1514;

use embassy_net::{Stack, StackResources};
use embassy_usb::class::cdc_ncm::embassy_net::{Device, Runner};

#[embassy_executor::task]
async fn usb_ncm_task(class: Runner<'static, UsbDriver, MTU>) -> ! {
    class.run().await
}

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<Device<'static, MTU>>) -> ! {
    stack.run().await
}
// usb net end
//

// #[cfg(context = "rp2040")]
// #[embassy_executor::task]
// async fn embassy_init(p: Peripherals) {
//     use embassy_rp::uart::{Config, UartTx};
//     use embassy_time::{Duration, Timer};
//     let mut uart_tx = UartTx::new(p.UART0, p.PIN_0, p.DMA_CH0, Config::default());

//     loop {
//         let data = b"hello\n";
//         uart_tx.write(&data[..]).await.unwrap();
//         Timer::after(Duration::from_secs(1)).await;
//     }
// }

const fn usb_default_config() -> embassy_usb::Config<'static> {
    // Create embassy-usb Config
    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("USB-Ethernet example");
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

pub(crate) fn init() {
    riot_rs_rt::debug::println!("riot-rs-embassy::init()");
    let p = arch::init(Default::default());
    EXECUTOR.start(SWI);

    EXECUTOR.spawner().spawn(init_task(p)).unwrap();
    riot_rs_rt::debug::println!("riot-rs-embassy::init() done");
}

#[embassy_executor::task]
async fn init_task(peripherals: arch::Peripherals) {
    riot_rs_rt::debug::println!("riot-rs-embassy::init_task()");
    #[cfg(all(context = "nrf52", feature = "usb"))]
    {
        // nrf52840
        let clock: embassy_nrf::pac::CLOCK = unsafe { core::mem::transmute(()) };

        riot_rs_rt::debug::println!("nrf: enabling ext hfosc...");
        clock.tasks_hfclkstart.write(|w| unsafe { w.bits(1) });
        while clock.events_hfclkstarted.read().bits() != 1 {}
    }

    #[cfg(context = "nrf52")]
    let usb_driver = nrf52::usb_driver(peripherals.USBD);

    #[cfg(context = "rp2040")]
    let usb_driver = rp2040::usb_driver(peripherals.USB);

    let usb_config = usb_default_config();

    // Create embassy-usb DeviceBuilder using the driver and config.
    let mut builder = Builder::new(
        usb_driver,
        usb_config,
        &mut make_static!([0; 256])[..],
        &mut make_static!([0; 256])[..],
        &mut make_static!([0; 256])[..],
        &mut make_static!([0; 128])[..],
        &mut make_static!([0; 128])[..],
    );

    // Our MAC addr.
    let our_mac_addr = [0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC];
    // Host's MAC addr. This is the MAC the host "thinks" its USB-to-ethernet adapter has.
    let host_mac_addr = [0x88, 0x88, 0x88, 0x88, 0x88, 0x88];

    let usb_cdc_ecm = {
        use embassy_usb::class::cdc_ncm::{CdcNcmClass, State};

        // Create classes on the builder.
        CdcNcmClass::new(&mut builder, make_static!(State::new()), host_mac_addr, 64)
    };

    let spawner = Spawner::for_current_executor().await;

    // Build the builder.
    let usb = builder.build();

    spawner.spawn(usb_task(usb)).unwrap();

    // let (runner, device) =
    //     class.into_embassy_net_device::<MTU, 4, 4>(make_static!(NetState::new()), our_mac_addr);
    // unwrap!(spawner.spawn(usb_ncm_task(runner)));

    use embassy_usb::class::cdc_ncm::embassy_net::State as NetState;
    let (runner, device) = usb_cdc_ecm
        .into_embassy_net_device::<MTU, 4, 4>(make_static!(NetState::new()), our_mac_addr);

    spawner.spawn(usb_ncm_task(runner)).unwrap();

    // network stack
    //let config = embassy_net::Config::dhcpv4(Default::default());
    use embassy_net::{Ipv4Address, Ipv4Cidr};
    let config = embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
        address: Ipv4Cidr::new(Ipv4Address::new(10, 42, 0, 61), 24),
        dns_servers: heapless::Vec::new(),
        gateway: Some(Ipv4Address::new(10, 42, 0, 1)),
    });

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
        make_static!(StackResources::<2>::new()),
        seed
    ));

    spawner.spawn(net_task(stack)).unwrap();

    riot_rs_rt::debug::println!("riot-rs-embassy::init_task() done");
}

use linkme::distributed_slice;
use riot_rs_rt::INIT_FUNCS;

#[distributed_slice(INIT_FUNCS)]
static RIOT_RS_EMBASSY_INIT: fn() = init;
