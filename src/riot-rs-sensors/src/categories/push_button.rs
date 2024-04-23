use core::future::Future;

use crate::{
    sensor::{PhysicalValue, ReadingResult},
    Sensor,
};

pub trait PushButtonSensor: Sensor {
    fn read_press_state(&self) -> impl Future<Output = ReadingResult<PushButtonReading>>;
}

#[derive(Debug)]
pub struct PushButtonReading {
    value: PhysicalValue,
}

impl PushButtonReading {
    pub fn new(value: PhysicalValue) -> Self {
        Self { value }
    }

    pub fn is_pressed(&self) -> bool {
        self.value.value() != 0
    }
}
