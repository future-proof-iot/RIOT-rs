use picoserve::response::IntoResponse;
use riot_rs::sensors::{Reading, Sensor, REGISTRY};

pub async fn sensors() -> impl IntoResponse {
    for sensor in REGISTRY.sensors() {
        let reading = riot_rs::read_sensor!(sensor);

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
