#[doc(alias = "master")]
pub mod controller;

pub fn init(peripherals: &mut crate::OptionalPeripherals) {
    // Take all I2C peripherals and do nothing with them.
    cfg_if::cfg_if! {
        if #[cfg(context = "rp2040")] {
            let _ = peripherals.I2C0.take().unwrap();
            let _ = peripherals.I2C1.take().unwrap();
        } else {
            compile_error!("this RP chip is not supported");
        }
    }
}
