use picoserve::response::{IntoResponse, Json};
use riot_rs::sensors::{categories::temperature::TemperatureSensor, sensor::PhysicalUnit, Sensor};

use crate::sensors::TEMP_SENSOR;

pub async fn temp() -> impl IntoResponse {
    let temp = TEMP_SENSOR
        .read_temperature()
        .await
        .unwrap()
        .temperature()
        .value();
    let unit = TEMP_SENSOR.unit();

    Json(JsonTemp { temp, unit })
}

#[derive(serde::Serialize)]
struct JsonTemp {
    // Temperature in hundredths of degrees Celsius
    temp: i32,
    unit: PhysicalUnit,
}
