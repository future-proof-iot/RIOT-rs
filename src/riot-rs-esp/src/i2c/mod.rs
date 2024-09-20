#[doc(alias = "master")]
pub mod controller;

pub fn init(peripherals: &mut crate::OptionalPeripherals) {
    // Take all I2C peripherals and do nothing with them.
    cfg_if::cfg_if! {
        if #[cfg(context = "esp32c6")] {
            let _ = peripherals.I2C0.take().unwrap();
        } else {
            compile_error!("this ESP32 chip is not supported");
        }
    }
}
