//! See your architecture's Embassy crate documentation.
use embassy_hal_internal::PeripheralRef;

pub struct Output<'d> {
    pub(crate) pin: Flex<'d>,
}

pub struct Flex<'d> {
    pub(crate) pin: PeripheralRef<'d, AnyPin>,
}

pub struct AnyPin {
    pin_port: u8,
}
