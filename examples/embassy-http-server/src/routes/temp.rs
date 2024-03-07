use fixed::traits::LossyInto;
use picoserve::{
    extract::State,
    response::{IntoResponse, Json},
};

use crate::TempInput;

pub async fn temp(State(TempInput(temp)): State<TempInput>) -> impl IntoResponse {
    let temp = (100 * temp.lock().await.read().await).lossy_into();

    Json(JsonTemp { temp })
}

#[derive(serde::Serialize)]
struct JsonTemp {
    // Temperature in hundredths of degrees Celsius
    temp: i32,
}
