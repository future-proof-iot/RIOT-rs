#[doc(alias = "master")]
pub mod controller;

#[doc(hidden)]
pub fn init(peripherals: &mut crate::OptionalPeripherals) {
    // This macro has to be defined in this function so that the `peripherals` variables exists.
    macro_rules! take_all_i2c_peripherals {
        ($peripherals:ident, $( $peripheral:ident ),*) => {
            $(
                let _ = peripherals.$peripheral.take().unwrap();
            )*
        }
    }

    // Take all I2c peripherals and do nothing with them.
    cfg_if::cfg_if! {
        if #[cfg(context = "stm32f401retx")] {
            take_all_i2c_peripherals!(I2C1, I2C2, I2C3);
        } else if #[cfg(context = "stm32h755zitx")] {
            take_all_i2c_peripherals!(I2C1, I2C2, I2C3, I2C4);
        } else if #[cfg(context = "stm32wb55rgvx")] {
            take_all_i2c_peripherals!(I2C1, I2C3);
        } else {
            compile_error!("this STM32 chip is not supported");
        }
    }
}
