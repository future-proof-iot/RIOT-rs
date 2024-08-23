use embassy_stm32::{
    bind_interrupts,
    i2c::{
        ErrorInterruptHandler, EventInterruptHandler, I2c as InnerI2c, RxDma, SclPin, SdaPin, TxDma,
    },
    mode::Async,
    peripherals,
    time::Hertz,
    Peripheral,
};
use embedded_hal_async::i2c::Operation;
use riot_rs_macros::call_with_stm32_peripheral_list;

use crate::{arch, i2c::impl_async_i2c_for_driver_enum};

#[non_exhaustive]
#[derive(Clone)]
pub struct Config {
    pub frequency: Frequency,
    pub sda_pullup: bool,
    pub scl_pullup: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frequency: Frequency::_100k,
            sda_pullup: false,
            scl_pullup: false,
        }
    }
}

// NOTE(arch): intermediate frequencies are also supported.
// TODO: also support arbitrary frequencies?
#[derive(Copy, Clone)]
#[repr(u32)]
pub enum Frequency {
    /// Standard mode.
    _100k = 100_000,
    _250k = 250_000,
    /// Fast mode.
    _400k = 400_000,
    // FIXME: frequencies up to 1 MHz are supported, but requires additional configuration of GPIOs
    // used.
}

impl From<Frequency> for Hertz {
    fn from(freq: Frequency) -> Self {
        match freq {
            Frequency::_100k => Hertz::khz(100),
            Frequency::_250k => Hertz::khz(250),
            Frequency::_400k => Hertz::khz(400),
        }
    }
}

pub(crate) fn init(peripherals: &mut arch::OptionalPeripherals) {
    // This macro has to be defined in this function so that the `peripherals` variables exists.
    macro_rules! take_all_i2c_peripherals {
        ($peripherals:ident, $( $peripheral:ident ),*) => {
            $(
                let _ = peripherals.$peripheral.take().unwrap();
            )*
        }
    }

    // Take all I2c peripherals and do nothing with them.
    call_with_stm32_peripheral_list!(take_all_i2c_peripherals!, I2c, Peripherals);
}

macro_rules! define_i2c_drivers {
    ($( $ev_interrupt:ident + $er_interrupt:ident => $peripheral:ident ),* $(,)?) => {
        $(
            /// Peripheral-specific I2C driver.
            // NOTE(arch): this is not required on this architecture, as the inner I2C type is
            // not generic over the I2C peripheral, and is only done for consistency with
            // other architectures.
            pub struct $peripheral {
                twim: InnerI2c<'static, Async>
            }

            impl $peripheral {
                #[must_use]
                pub fn new(
                    sda_pin: impl Peripheral<P: SdaPin<peripherals::$peripheral>> + 'static,
                    scl_pin: impl Peripheral<P: SclPin<peripherals::$peripheral>> + 'static,
                    tx_dma: impl Peripheral<P: TxDma<peripherals::$peripheral>> + 'static,
                    rx_dma: impl Peripheral<P: RxDma<peripherals::$peripheral>> + 'static,
                    config: Config,
                ) -> I2c {
                    let mut i2c_config = embassy_stm32::i2c::Config::default();
                    i2c_config.sda_pullup = config.sda_pullup;
                    i2c_config.scl_pullup = config.scl_pullup;
                    i2c_config.timeout = crate::i2c::I2C_TIMEOUT;

                    bind_interrupts!(
                        struct Irqs {
                            $ev_interrupt => EventInterruptHandler<peripherals::$peripheral>;
                            $er_interrupt => ErrorInterruptHandler<peripherals::$peripheral>;
                        }
                    );

                    // Make this struct a compile-time-enforced singleton: having multiple statics
                    // defined with the same name would result in a compile-time error.
                    paste::paste! {
                        #[allow(dead_code)]
                        static [<PREVENT_MULTIPLE_ $peripheral>]: () = ();
                    }

                    // FIXME(safety): enforce that the init code indeed has run
                    // SAFETY: this struct being a singleton prevents us from stealing the
                    // peripheral multiple times.
                    let twim_peripheral = unsafe { peripherals::$peripheral::steal() };

                    let frequency = config.frequency;
                    let i2c = InnerI2c::new(
                        twim_peripheral,
                        scl_pin,
                        sda_pin,
                        Irqs,
                        tx_dma,
                        rx_dma,
                        frequency.into(),
                        i2c_config,
                    );

                    I2c::$peripheral(Self { twim: i2c })
                }
            }
        )*

        /// Peripheral-agnostic driver.
        pub enum I2c {
            $( $peripheral($peripheral), )*
        }

        impl embedded_hal_async::i2c::ErrorType for I2c {
            type Error = crate::i2c::Error;
        }

        impl_async_i2c_for_driver_enum!(I2c, $( $peripheral ),*);
    }
}

impl From<embassy_stm32::i2c::Error> for crate::i2c::Error {
    fn from(err: embassy_stm32::i2c::Error) -> Self {
        use embassy_stm32::i2c::Error::*;

        use crate::i2c::{Error, NoAcknowledgeSource};

        match err {
            Bus => Error::Bus,
            Arbitration => Error::ArbitrationLoss,
            Nack => Error::NoAcknowledge(NoAcknowledgeSource::Unknown),
            Timeout => Error::Timeout,
            Crc => Error::Other,
            Overrun => Error::Overrun,
            ZeroLengthTransfer => Error::Other,
        }
    }
}

// Define a driver per peripheral
call_with_stm32_peripheral_list!(define_i2c_drivers!, I2c, PeripheralsAndInterrupts);
