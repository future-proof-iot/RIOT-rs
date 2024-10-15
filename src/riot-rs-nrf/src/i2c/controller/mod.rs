use embassy_nrf::{
    bind_interrupts,
    gpio::Pin as GpioPin,
    peripherals,
    twim::{InterruptHandler, Twim},
    Peripheral,
};
use riot_rs_embassy_common::impl_async_i2c_for_driver_enum;

/// I2C bus configuration.
#[non_exhaustive]
#[derive(Clone)]
pub struct Config {
    pub frequency: Frequency,
    pub sda_pullup: bool,
    pub scl_pullup: bool,
    pub sda_high_drive: bool,
    pub scl_high_drive: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frequency: Frequency::_100k,
            sda_pullup: false,
            scl_pullup: false,
            sda_high_drive: false,
            scl_high_drive: false,
        }
    }
}

/// I2C bus frequency.
// NOTE(arch): the datasheets only mention these frequencies.
#[cfg(any(context = "nrf52833", context = "nrf52840", context = "nrf5340"))]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Frequency {
    /// Standard mode.
    _100k,
    #[cfg(any(context = "nrf52833", context = "nrf5340"))]
    _250k,
    /// Fast mode.
    _400k,
    // FIXME(embassy): the upstream Embassy crate does not support this frequency
    // #[cfg(context = "nrf5340")]
    // _1M,
}

impl Frequency {
    pub const fn first() -> Self {
        Self::_100k
    }

    pub const fn last() -> Self {
        Self::_400k
    }

    pub const fn next(self) -> Option<Self> {
        match self {
            #[cfg(context = "nrf52840")]
            Self::_100k => Some(Self::_400k),
            #[cfg(any(context = "nrf52833", context = "nrf5340"))]
            Self::_100k => Some(Self::_250k),
            #[cfg(any(context = "nrf52833", context = "nrf5340"))]
            Self::_250k => Some(Self::_400k),
            Self::_400k => None,
        }
    }

    pub const fn prev(self) -> Option<Self> {
        match self {
            Self::_100k => None,
            #[cfg(any(context = "nrf52833", context = "nrf5340"))]
            Self::_250k => Some(Self::_100k),
            #[cfg(context = "nrf52840")]
            Self::_400k => Some(Self::_100k),
            #[cfg(any(context = "nrf52833", context = "nrf5340"))]
            Self::_400k => Some(Self::_250k),
        }
    }

    pub const fn khz(self) -> u32 {
        match self {
            Self::_100k => 100,
            #[cfg(any(context = "nrf52833", context = "nrf5340"))]
            Self::_250k => 250,
            Self::_400k => 400,
        }
    }
}

riot_rs_embassy_common::impl_i2c_from_frequency!();

impl From<Frequency> for embassy_nrf::twim::Frequency {
    fn from(freq: Frequency) -> Self {
        match freq {
            Frequency::_100k => embassy_nrf::twim::Frequency::K100,
            #[cfg(any(context = "nrf52833", context = "nrf5340"))]
            Frequency::_250k => embassy_nrf::twim::Frequency::K250,
            Frequency::_400k => embassy_nrf::twim::Frequency::K400,
        }
    }
}

macro_rules! define_i2c_drivers {
    ($( $interrupt:ident => $peripheral:ident ),* $(,)?) => {
        $(
            /// Peripheral-specific I2C driver.
            pub struct $peripheral {
                twim: Twim<'static, peripherals::$peripheral>,
            }

            impl $peripheral {
                #[expect(clippy::new_ret_no_self)]
                #[must_use]
                pub fn new(
                    sda_pin: impl Peripheral<P: GpioPin> + 'static,
                    scl_pin: impl Peripheral<P: GpioPin> + 'static,
                    config: Config,
                ) -> I2c {
                    let mut twim_config = embassy_nrf::twim::Config::default();
                    twim_config.frequency = config.frequency.into();
                    twim_config.sda_pullup = config.sda_pullup;
                    twim_config.scl_pullup = config.scl_pullup;
                    twim_config.sda_high_drive = config.sda_high_drive;
                    twim_config.scl_high_drive = config.scl_high_drive;

                    bind_interrupts!(
                        struct Irqs {
                            $interrupt => InterruptHandler<peripherals::$peripheral>;
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

                    // NOTE(arch): the I2C peripheral and driver do not have any built-in timeout,
                    // we implement it at a higher level, not in this arch-specific module.
                    let twim = Twim::new(twim_peripheral, Irqs, sda_pin, scl_pin, twim_config);

                    I2c::$peripheral(Self { twim })
                }
            }
        )*

        /// Peripheral-agnostic driver.
        pub enum I2c {
            $( $peripheral($peripheral), )*
        }

        impl embedded_hal_async::i2c::ErrorType for I2c {
            type Error = riot_rs_embassy_common::i2c::controller::Error;
        }

        impl_async_i2c_for_driver_enum!(I2c, $( $peripheral ),*);
    }
}

// We cannot impl From because both types are external to this crate.
fn from_error(err: embassy_nrf::twim::Error) -> riot_rs_embassy_common::i2c::controller::Error {
    use embassy_nrf::twim::Error::*;

    use riot_rs_embassy_common::i2c::controller::{Error, NoAcknowledgeSource};

    match err {
        TxBufferTooLong => Error::Other,
        RxBufferTooLong => Error::Other,
        Transmit => Error::Other,
        Receive => Error::Other,
        BufferNotInRAM => Error::Other,
        AddressNack => Error::NoAcknowledge(NoAcknowledgeSource::Address),
        DataNack => Error::NoAcknowledge(NoAcknowledgeSource::Data),
        Overrun => Error::Overrun,
        Timeout => Error::Timeout,
        _ => Error::Other,
    }
}

// FIXME: support other nRF archs
// Define a driver per peripheral
#[cfg(any(context = "nrf52833", context = "nrf52840"))]
define_i2c_drivers!(
    SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0 => TWISPI0,
    SPIM1_SPIS1_TWIM1_TWIS1_SPI1_TWI1 => TWISPI1,
);
#[cfg(context = "nrf5340")]
define_i2c_drivers!(
    SERIAL0 => SERIAL0,
    SERIAL1 => SERIAL1,
);
