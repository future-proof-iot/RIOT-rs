pub(crate) use embassy_executor::InterruptExecutor as Executor;

#[cfg(context = "nrf52")]
pub use embassy_nrf::interrupt::SWI0_EGU0 as SWI;

#[cfg(context = "nrf5340")]
pub use embassy_nrf::interrupt::EGU0 as SWI;

pub use embassy_nrf::{config::Config, interrupt, peripherals, OptionalPeripherals};

#[cfg(context = "nrf52")]
#[interrupt]
unsafe fn SWI0_EGU0() {
    // SAFETY:
    // - called from ISR
    // - not called before `start()`, as the interrupt is enabled by `start()`
    //   itself
    unsafe { crate::EXECUTOR.on_interrupt() }
}

#[cfg(context = "nrf5340")]
#[interrupt]
unsafe fn EGU0() {
    unsafe { crate::EXECUTOR.on_interrupt() }
}

#[cfg(feature = "usb")]
pub mod usb {
    use embassy_nrf::{
        bind_interrupts, peripherals,
        usb::{
            self,
            vbus_detect::{self, HardwareVbusDetect},
            Driver,
        },
    };

    use crate::arch;

    #[cfg(context = "nrf52")]
    bind_interrupts!(struct Irqs {
        USBD => usb::InterruptHandler<peripherals::USBD>;
        POWER_CLOCK => vbus_detect::InterruptHandler;
    });

    #[cfg(context = "nrf5340")]
    bind_interrupts!(struct Irqs {
        USBD => usb::InterruptHandler<peripherals::USBD>;
        USBREGULATOR => vbus_detect::InterruptHandler;
    });

    pub type UsbDriver = Driver<'static, peripherals::USBD, HardwareVbusDetect>;

    pub fn driver(peripherals: &mut arch::OptionalPeripherals) -> UsbDriver {
        let usbd = peripherals.USBD.take().unwrap();
        Driver::new(usbd, Irqs, HardwareVbusDetect::new(Irqs))
    }
}

pub fn init(config: Config) -> OptionalPeripherals {
    let peripherals = embassy_nrf::init(config);
    OptionalPeripherals::from(peripherals)
}

#[cfg(feature = "internal-temp")]
pub mod internal_temp {
    // FIXME: maybe use portable_atomic's instead
    use core::sync::atomic::{AtomicBool, Ordering};

    use embassy_nrf::{peripherals, temp};
    use embassy_sync::{
        blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, mutex::Mutex,
    };
    use riot_rs_saga::sensor::{
        Notification, NotificationReceiver, PhysicalUnit, PhysicalValue, Reading, ReadingError,
        ReadingResult, Sensor,
    };

    embassy_nrf::bind_interrupts!(struct Irqs {
        TEMP => embassy_nrf::temp::InterruptHandler;
    });

    pub struct InternalTemp {
        enabled: AtomicBool,
        temp: Mutex<CriticalSectionRawMutex, Option<temp::Temp<'static>>>,
        channel: Channel<CriticalSectionRawMutex, Notification, 1>,
    }

    impl InternalTemp {
        pub const fn new() -> Self {
            Self {
                enabled: AtomicBool::new(false),
                temp: Mutex::new(None),
                channel: Channel::new(),
            }
        }

        pub fn init(&self, peripheral: peripherals::TEMP) {
            if !self.enabled.load(Ordering::Acquire) {
                // FIXME: we use try_lock instead of lock to not make this function async, can we do
                // better?
                // FIXME: return an error when relevant
                let mut temp = self.temp.try_lock().unwrap();
                *temp = Some(temp::Temp::new(peripheral, Irqs));

                // fn thread_fn() {
                //     riot_rs_debug::println!("Thread started");
                //     loop {}
                //     // loop {
                //     //     if let Ok(value) = sensor.read() {
                //     //         // FIXME: use the value set with set_lower_threshold()
                //     //         if value.value > 22 {
                //     //             // FIXME: should this be LowerThreshold or HigherThreshold?
                //     //             let _ = sensor.channel.send(Notification::LowerThreshold);
                //     //             riot_rs_debug::println!("Test");
                //     //         }
                //     //     }
                //     //
                //     //     // FIXME: do not busy loop, sleep for some time
                //     // }
                // }

                // let mut stack = [0u8; 2048_usize];
                // riot_rs_threads::thread_create_noarg(thread_fn, &mut stack, 1);
                // riot_rs_debug::println!("Test", );

                self.enabled.store(true, Ordering::Release);
            }
        }
    }

    // pub struct TemperatureReading(PhysicalValue);
    //
    // impl Reading for TemperatureReading {
    //     fn value(&self) -> PhysicalValue {
    //         self.0
    //     }
    // }

    impl Sensor for InternalTemp {
        fn read(&self) -> ReadingResult<PhysicalValue> {
            use fixed::traits::LossyInto;

            if !self.enabled.load(Ordering::Acquire) {
                return Err(ReadingError::Disabled);
            }

            let reading = embassy_futures::block_on(async {
                self.temp.lock().await.as_mut().unwrap().read().await
            });

            let temp: i32 = (100 * reading).lossy_into();

            Ok(PhysicalValue { value: temp })
        }

        fn enabled(&self) -> bool {
            self.enabled.load(Ordering::Acquire)
        }

        fn set_lower_threshold(&self, value: PhysicalValue) {
            // FIXME
        }

        fn subscribe(&self) -> NotificationReceiver {
            // TODO: receiver competes for notification: limit the number of receivers to 1?
            self.channel.receiver()
        }

        fn value_scale() -> i8 {
            -2
        }

        fn unit() -> PhysicalUnit {
            PhysicalUnit::Celsius
        }

        fn display_name() -> Option<&'static str> {
            Some("Internal temperature sensor")
        }

        fn part_number() -> &'static str {
            "nrf52 internal temperature sensor"
        }

        fn version() -> u8 {
            0
        }
    }
}
