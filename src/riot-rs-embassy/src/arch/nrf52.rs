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
    use core::sync::atomic::{AtomicBool, AtomicI32, Ordering};

    use embassy_executor::Spawner;
    use embassy_nrf::{peripherals, temp};
    use embassy_sync::{
        blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, mutex::Mutex,
    };
    use embassy_time::{Duration, Timer};
    use riot_rs_sensors::sensor::{
        Notification, NotificationReceiver, PhysicalUnit, PhysicalValue, Reading, ReadingError,
        ReadingResult, Sensor, ThresholdKind,
    };

    embassy_nrf::bind_interrupts!(struct Irqs {
        TEMP => embassy_nrf::temp::InterruptHandler;
    });

    pub struct InternalTemp {
        initialized: AtomicBool, // TODO: use an atomic bitset for initialized and enabled
        enabled: AtomicBool,
        temp: Mutex<CriticalSectionRawMutex, Option<temp::Temp<'static>>>,
        channel: Channel<CriticalSectionRawMutex, Notification, 1>,
        // feature is not used
        lower_threshold: AtomicI32,
        lower_threshold_enabled: AtomicBool, // TODO: use an atomic bitset for handler other
                                             // thresholds
    }

    impl InternalTemp {
        pub const fn new() -> Self {
            Self {
                initialized: AtomicBool::new(false),
                enabled: AtomicBool::new(false),
                temp: Mutex::new(None),
                channel: Channel::new(),
                lower_threshold: AtomicI32::new(0),
                lower_threshold_enabled: AtomicBool::new(false),
            }
        }

        pub fn init(&'static self, spawner: Spawner, peripheral: peripherals::TEMP) {
            if !self.initialized.load(Ordering::Acquire) {
                // We use `try_lock()` instead of `lock()` to not make this function async.
                // This mutex cannot be locked at this point as it is private and can only be
                // locked when the sensor has been initialized successfully.
                let mut temp = self.temp.try_lock().unwrap();
                *temp = Some(temp::Temp::new(peripheral, Irqs));

                #[embassy_executor::task]
                async fn temp_watcher(sensor: &'static InternalTemp) {
                    loop {
                        if sensor.lower_threshold_enabled.load(Ordering::Acquire) {
                            if let Ok(value) = sensor.read().await {
                                if value.value().value()
                                    > sensor.lower_threshold.load(Ordering::Acquire)
                                {
                                    // FIXME: should this be Lower or Higher?
                                    let _ = sensor
                                        .channel
                                        .try_send(Notification::Threshold(ThresholdKind::Lower));
                                    riot_rs_debug::println!("Temp > lower threshold: {:?}", value);
                                }
                            }
                        }
                        // TODO: make this duration configurable?
                        // Avoid busy looping and allow other users to lock the mutex
                        Timer::after(Duration::from_millis(100)).await;
                    }
                }
                spawner.spawn(temp_watcher(&self)).unwrap();

                self.initialized.store(true, Ordering::Release);
                self.enabled.store(true, Ordering::Release);
            }
        }
    }

    #[derive(Debug)]
    pub struct TemperatureReading(PhysicalValue);

    impl Reading for TemperatureReading {
        fn value(&self) -> PhysicalValue {
            self.0
        }
    }

    impl Sensor for InternalTemp {
        async fn read(&self) -> ReadingResult<TemperatureReading> {
            use fixed::traits::LossyInto;

            if !self.enabled.load(Ordering::Acquire) {
                return Err(ReadingError::Disabled);
            }

            let reading = self.temp.lock().await.as_mut().unwrap().read().await;
            let temp: i32 = (100 * reading).lossy_into();

            Ok(TemperatureReading(PhysicalValue::new(temp)))
        }

        fn set_enabled(&self, enabled: bool) {
            if self.initialized.load(Ordering::Acquire) {
                self.enabled.store(enabled, Ordering::Release);
            }
            // TODO: return an error otherwise?
        }

        fn enabled(&self) -> bool {
            self.enabled.load(Ordering::Acquire)
        }

        fn set_threshold(&self, kind: ThresholdKind, value: PhysicalValue) {
            match kind {
                ThresholdKind::Lower => {
                    self.lower_threshold.store(value.value(), Ordering::Release)
                }
                _ => {
                    // TODO: should we return an error instead?
                }
            }
        }

        fn set_threshold_enabled(&self, kind: ThresholdKind, enabled: bool) {
            match kind {
                ThresholdKind::Lower => self
                    .lower_threshold_enabled
                    .store(enabled, Ordering::Release),
                _ => {
                    // TODO: should we return an error instead?
                }
            }
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
