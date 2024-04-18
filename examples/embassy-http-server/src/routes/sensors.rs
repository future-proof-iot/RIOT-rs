use picoserve::response::IntoResponse;
use riot_rs::sensors::{Sensor, REGISTRY};

pub async fn sensors() -> impl IntoResponse {
    for sensor in REGISTRY.sensors() {
        let reading = riot_rs::await_read_sensor_main!(sensor);

        if let Ok(value) = reading {
            riot_rs::debug::println!("{:?}", value.value());
        } else {
            return "Error reading sensor";
        }
    }

    "No sensors"
}

#[derive(serde::Serialize)]
struct JsonTemp {
    // Temperature in hundredths of degrees Celsius
    temp: i32,
}
