use picoserve::response::{IntoResponse, Json};
use riot_rs::sensors::categories::temperature::TemperatureSensor;

pub async fn temp() -> impl IntoResponse {
    let temp = crate::sensors::TEMP_SENSOR
        .read_temperature()
        .await
        .unwrap()
        .temperature()
        .value();

    Json(JsonTemp { temp })
}

#[derive(serde::Serialize)]
struct JsonTemp {
    // Temperature in hundredths of degrees Celsius
    temp: i32,
}
