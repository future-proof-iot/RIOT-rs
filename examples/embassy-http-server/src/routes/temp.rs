use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use picoserve::{
    extract::State,
    response::{IntoResponse, Json},
};
use riot_rs::sensors::sensor::{Reading, Sensor};

use crate::{println, TEMP_SENSOR};

pub async fn temp() -> impl IntoResponse {
    // let signal: Signal<CriticalSectionRawMutex, i32> = Signal::new();
    //
    // fn read_temp(signal: &Signal<CriticalSectionRawMutex, i32>) {
    //     // FIXME: handle this unwrap
    //     let temp = TEMP_SENSOR.read().unwrap().value;
    //     signal.signal(temp);
    // }
    //
    // let mut stack = make_static!([0u8; 4096_usize]);
    // thread::thread_create(read_temp, &signal, &mut stack, 1);
    //
    // let temp = signal.wait().await;

    let temp = TEMP_SENSOR.read().await.unwrap().value().value;

    Json(JsonTemp { temp })
}

#[derive(serde::Serialize)]
struct JsonTemp {
    // Temperature in hundredths of degrees Celsius
    temp: i32,
}
