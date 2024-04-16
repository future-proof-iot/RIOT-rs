use core::future::Future;

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use heapless::Vec;
use static_cell::StaticCell;

use crate::sensor::{PhysicalUnit, PhysicalValue, ReadingResult, Sensor};

#[linkme::distributed_slice]
pub static SENSOR_REFS: [&'static dyn Sensor] = [..];

// static SENSORS: Mutex<CriticalSectionRawMutex, Vec<&'static dyn Sensor, 8>> =
// Mutex::new(Vec::new());

pub static REGISTRY: Registry = Registry::new();

pub struct Registry {
    // sensors:
    //     Mutex<CriticalSectionRawMutex, [StaticCell<dyn Sensor>; 8]>, // FIXME: use an env var or something
}

impl Registry {
    const fn new() -> Self {
        Self {
            // sensors: Mutex::new(Vec::new()),
        }
    }

    // pub fn register(&self, sensor: impl Sensor) -> &'static impl Sensor {
    //     for slot in self.sensors.try_lock().unwrap() {
    //         let Some(sensor_ref) = slot.try_init(sensor) else {
    //             // TODO: return an error if already full
    //             unimplemented!()
    //         };
    //         // FIXME: do not unwrap in case the mutex is locked
    //         // FIXME: do something
    //         let _ = self.sensors.try_lock().unwrap().push(sensor_ref);
    //         return sensor_ref;
    //     }
    //     unreachable!();
    // }

    // pub async fn sensors(&self) -> &[&'static Mutex<CriticalSectionRawMutex, dyn Sensor>] {
    //     self.sensors.lock().await
    // }

    pub fn sensors(&self) -> impl Iterator<Item = &'static dyn Sensor> {
        // Returning an iterator instead of the distributed slice directly would allow us to chain
        // another source of sensors in the future, if we decided to support dynamically-allocated
        // sensors
        SENSOR_REFS.iter().copied()
    }

    // TODO: returns an iterator returning async values, do we want to asynchronously return an
    // iterator instead, which would ready every sensor concurrently?
    // pub async fn read_all(&self) -> ReadAll {
    //     ReadAll { sensor_index: 0 }
    // }
}
