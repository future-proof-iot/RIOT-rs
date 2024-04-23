use core::future::Future;

use crate::{
    sensor::{PhysicalValue, ReadingResult},
    Sensor,
};

pub trait TemperatureSensor: Sensor {
    fn read_temperature(&self) -> impl Future<Output = ReadingResult<TemperatureReading>>;
}

#[derive(Debug)]
pub struct TemperatureReading {
    value: PhysicalValue,
}

impl TemperatureReading {
    pub fn new(value: PhysicalValue) -> Self {
        Self { value }
    }

    pub fn temperature(&self) -> PhysicalValue {
        self.value
    }
}
