//! This example is merely to illustrate and test raw bus usage.
//!
//! Please use [`riot_rs::sensors`] instead for a high-level sensor abstraction that is
//! architecture-agnostic.
//!
//! This example requires a LIS3DH sensor (3-axis accelerometer).
#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use embassy_sync::mutex::Mutex;
use embedded_hal_async::i2c::I2c as _;
use riot_rs::{
    debug::{exit, println, EXIT_SUCCESS},
    embassy::{
        arch::{i2c, peripherals},
        i2c::I2cDevice,
    },
};

const LIS3DH_I2C_ADDR: u8 = 0x19;

// WHO_AM_I register of the LIS3DH sensor
const WHO_AM_I_REG_ADDR: u8 = 0x0f;

#[cfg(context = "rp")]
riot_rs::define_peripherals!(Peripherals {
    i2c_peripheral: I2C0,
    i2c_sda: PIN_12,
    i2c_scl: PIN_13,
});

#[cfg(context = "st-nucleo-wb55")]
riot_rs::define_peripherals!(Peripherals {
    i2c_peripheral: I2C1,
    i2c_sda: PB9,
    i2c_scl: PB8,
    i2c_tx_dma: DMA1_CH1,
    i2c_rx_dma: DMA1_CH2,
});

pub static I2C_BUS: once_cell::sync::OnceCell<
    Mutex<embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex, i2c::I2c>,
> = once_cell::sync::OnceCell::new();

#[riot_rs::task(autostart, peripherals)]
async fn main(peripherals: Peripherals) {
    let mut i2c_config = i2c::Config::default();
    i2c_config.frequency = i2c::Frequency::K100;

    #[cfg(context = "rp")]
    let i2c_bus = i2c::I2c::I2C0(i2c::I2cI2C0::new(
        peripherals.i2c_peripheral,
        peripherals.i2c_sda,
        peripherals.i2c_scl,
        i2c_config,
    ));

    #[cfg(context = "stm32wb55rgvx")]
    let i2c_bus = i2c::I2c::I2C1(i2c::I2cI2C1::new(
        peripherals.i2c_peripheral,
        peripherals.i2c_sda,
        peripherals.i2c_scl,
        peripherals.i2c_tx_dma,
        peripherals.i2c_rx_dma,
        i2c_config,
    ));

    let _ = I2C_BUS.set(Mutex::new(i2c_bus));

    let mut i2c_device = I2cDevice::new(I2C_BUS.get().unwrap());

    let mut id = [0];
    i2c_device
        .write_read(LIS3DH_I2C_ADDR, &[WHO_AM_I_REG_ADDR], &mut id)
        .await
        .unwrap();

    let who_am_i = id[0];
    println!("LIS3DH WHO_AM_I_COMMAND register value: 0x{:x}", who_am_i);
    assert_eq!(who_am_i, 0x33);

    exit(EXIT_SUCCESS);
}
