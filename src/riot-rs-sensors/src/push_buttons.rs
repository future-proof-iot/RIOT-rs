// FIXME: use portable_atomic's instead
use core::sync::atomic::{AtomicBool, Ordering};

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, mutex::Mutex};
use embedded_hal::digital::InputPin;

use crate::{
    sensor::{
        Notification, NotificationReceiver, PhysicalUnit, PhysicalValue, ReadingError,
        ReadingResult, ThresholdKind,
    },
    Reading, Sensor,
};

pub struct PushButton<I: InputPin> {
    initialized: AtomicBool, // TODO: use an atomic bitset for initialized and enabled
    enabled: AtomicBool,
    // buttons: [Option<Button>; N], // TODO: maybe use MaybeUninit
    button: Mutex<CriticalSectionRawMutex, Option<I>>, // TODO: maybe use MaybeUninit
    channel: Channel<CriticalSectionRawMutex, Notification, 1>,
}

impl<I: InputPin> PushButton<I> {
    pub const fn new() -> Self {
        Self {
            initialized: AtomicBool::new(false),
            enabled: AtomicBool::new(false),
            button: Mutex::new(None),
            channel: Channel::new(),
        }
    }

    // TODO: add Spawner for consistency
    pub fn init(&'static self, gpio: I) {
        if !self.initialized.load(Ordering::Acquire) {
            // We use `try_lock()` instead of `lock()` to not make this function async.
            // This mutex cannot be locked at this point as it is private and can only be
            // locked when the sensor has been initialized successfully.
            let mut button = self.button.try_lock().unwrap();
            *button = Some(gpio);

            self.initialized.store(true, Ordering::Release);
            self.enabled.store(true, Ordering::Release);
        }
    }
}

#[derive(Debug)]
pub struct PushButtonReading(PhysicalValue);

impl Reading for PushButtonReading {
    fn value(&self) -> PhysicalValue {
        self.0
    }
}

impl PushButtonReading {
    pub fn is_pressed(&self) -> bool {
        self.0.value() != 0
    }
}

impl<I: 'static + InputPin + Send> Sensor for PushButton<I> {
    async fn read_main(&self) -> ReadingResult<PhysicalValue> {
        self.read().await.map(|v| v.value())
    }

    #[allow(refining_impl_trait)]
    async fn read(&self) -> ReadingResult<PushButtonReading> {
        if !self.enabled.load(Ordering::Acquire) {
            return Err(ReadingError::Disabled);
        }

        let reading = self.button.lock().await.as_mut().unwrap().is_low().unwrap();

        // FIXME: this has to be configurable to handle both active-low and active-high push button
        // inputs
        let is_pressed = reading;

        Ok(PushButtonReading(PhysicalValue::new(is_pressed as i32)))
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

    fn set_threshold(&self, _kind: ThresholdKind, _value: PhysicalValue) {}

    fn set_threshold_enabled(&self, _kind: ThresholdKind, _enabled: bool) {}

    fn subscribe(&self) -> NotificationReceiver {
        // TODO: receiver competes for notification: limit the number of receivers to 1?
        self.channel.receiver()
    }

    fn value_scale() -> i8 {
        0
    }

    fn unit() -> PhysicalUnit {
        PhysicalUnit::Bool
    }

    fn display_name() -> Option<&'static str> {
        Some("Push button")
    }

    fn part_number() -> &'static str {
        "push button"
    }

    fn version() -> u8 {
        0
    }
}
