use core::future::Future;

use picoserve::{
    extract::State,
    response::{IntoResponse, Json},
};
use riot_rs::{
    embassy::arch::internal_temp::InternalTemp,
    sensors::{
        registry::REGISTRY,
        sensor::{PhysicalValue, Reading, ReadingResult, Sensor},
    },
};

pub async fn sensors() -> impl IntoResponse {
    // riot_rs::rt::println!("{:?}", REGISTRY.sensors()[0].type_id());
    // for sensor in REGISTRY.sensors() {
    //     if let Some(sensor) = (*sensor as &dyn Any).downcast_ref::<InternalTemp>() {
    //         if let Ok(value) = sensor.read().await {
    // FIXME: seems to be stalling
    for reading in ReadAll::new() {
        if let Ok(value) = reading.await {
            riot_rs::debug::println!("{:?}", value.value());
            return "";
        }

        return "Error reading internal temp sensor";
    }

    "No sensors"
}

#[derive(serde::Serialize)]
struct JsonTemp {
    // Temperature in hundredths of degrees Celsius
    temp: i32,
}

pub struct ReadAll {
    sensor_index: usize,
}

impl ReadAll {
    #[must_use]
    fn new() -> Self {
        Self { sensor_index: 0 }
    }
}

impl Iterator for ReadAll {
    type Item = impl Future<Output = ReadingResult<impl Reading>>;

    fn next(&mut self) -> Option<<ReadAll as Iterator>::Item> {
        let sensor = REGISTRY.sensors().nth(self.sensor_index)?;
        self.sensor_index += 1;

        // TODO: codegen this based on the list of sensors
        // As `read()` is non-dispatchable, we have to downcast
        if let Some(sensor) = (sensor as &dyn core::any::Any).downcast_ref::<InternalTemp>() {
            return Some(sensor.read());
        }

        unimplemented!()

        // Some(async { sensor.read() })
    }
}
