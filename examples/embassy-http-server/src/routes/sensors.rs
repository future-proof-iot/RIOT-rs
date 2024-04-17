use picoserve::response::IntoResponse;
use riot_rs::sensors::{read_sensor, Reading, Sensor, REGISTRY};

pub async fn sensors() -> impl IntoResponse {
    for sensor in REGISTRY.sensors() {
        // TODO: codegen the list of sensors from the board configuration file
        // FIXME: use $crate if possible
        let reading = read_sensor!(sensor, riot_rs::embassy::arch::internal_temp::InternalTemp,);

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
