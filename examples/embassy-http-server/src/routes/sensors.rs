use core::any::Any;

use picoserve::{
    extract::State,
    response::{IntoResponse, Json},
};
use riot_rs::sensors::{
    registry::REGISTRY,
    sensor::{Reading, Sensor},
};

use crate::arch::internal_temp::InternalTemp;

pub async fn sensors() -> impl IntoResponse {
    // riot_rs::rt::println!("{:?}", REGISTRY.sensors()[0].type_id());
    // for sensor in REGISTRY.sensors() {
    //     if let Some(sensor) = (*sensor as &dyn Any).downcast_ref::<InternalTemp>() {
    //         if let Ok(value) = sensor.read().await {
    for reading in REGISTRY.read_all().await {
        if let Ok(value) = reading.await {
            riot_rs::debug::println!("{:?}", value.value);
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
