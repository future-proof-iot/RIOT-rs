//! This example is merely to illustrate and test raw bus usage.
//!
//! Please use [`ariel_os::sensors`] instead for a high-level sensor abstraction that is
//! HAL-agnostic.
//!
//! This example requires a LIS3DH/LSM303AGR sensor (3-axis accelerometer).
#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]
#![feature(impl_trait_in_assoc_type)]

mod pins;

use ariel_os::{
    debug::{
        exit,
        log::{debug, info},
        EXIT_SUCCESS,
    },
    hal,
    i2c::controller::{highest_freq_in, I2cDevice, Kilohertz},
};
use embassy_sync::mutex::Mutex;
use embedded_hal_async::i2c::I2c as _;

const TARGET_I2C_ADDR: u8 = 0x19;

// WHO_AM_I register of the sensor
const WHO_AM_I_REG_ADDR: u8 = 0x0f;

pub static I2C_BUS: once_cell::sync::OnceCell<
    Mutex<embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex, hal::i2c::controller::I2c>,
> = once_cell::sync::OnceCell::new();

#[ariel_os::task(autostart, peripherals)]
async fn main(peripherals: pins::Peripherals) {
    let mut i2c_config = hal::i2c::controller::Config::default();
    i2c_config.frequency = const { highest_freq_in(Kilohertz::kHz(100)..=Kilohertz::kHz(400)) };
    debug!("Selected frequency: {}", i2c_config.frequency);

    let i2c_bus = pins::SensorI2c::new(peripherals.i2c_sda, peripherals.i2c_scl, i2c_config);

    let _ = I2C_BUS.set(Mutex::new(i2c_bus));

    let mut i2c_device = I2cDevice::new(I2C_BUS.get().unwrap());

    let mut id = [0];
    i2c_device
        .write_read(TARGET_I2C_ADDR, &[WHO_AM_I_REG_ADDR], &mut id)
        .await
        .unwrap();

    let who_am_i = id[0];
    info!("WHO_AM_I_COMMAND register value: 0x{:x}", who_am_i);
    assert_eq!(who_am_i, 0x33);

    info!("Test passed!");

    exit(EXIT_SUCCESS);
}
