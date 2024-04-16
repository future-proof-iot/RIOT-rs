use picoserve::response::{IntoResponse, Json};
use riot_rs::sensors::{Reading, Sensor};

pub async fn temp() -> impl IntoResponse {
    let temp = crate::TEMP_SENSOR.read().await.unwrap().value().value();

    Json(JsonTemp { temp })
}

#[derive(serde::Serialize)]
struct JsonTemp {
    // Temperature in hundredths of degrees Celsius
    temp: i32,
}
