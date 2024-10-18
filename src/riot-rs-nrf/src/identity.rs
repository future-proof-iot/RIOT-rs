#[derive(Debug, defmt::Format)]
pub struct DeviceId(u64);

impl riot_rs_embassy_common::identity::DeviceId for DeviceId {
    type Error = core::convert::Infallible;

    fn get() -> Result<Self, Self::Error> {
        // Embassy does not wrap the FICR register, and given that all we need from there is a register
        // read that is perfectly fine to do through a stolen register, let's do that rather than
        // thread the access through several layers.

        // SAFETY: The register is used for read-only operations on constant values.
        #[cfg(context = "nrf52840")]
        let ficr = unsafe { nrf52840_pac::Peripherals::steal().FICR };
        #[cfg(context = "nrf52832")]
        let ficr = unsafe { nrf52832_pac::Peripherals::steal().FICR };
        #[cfg(context = "nrf5340")]
        let ficr = &unsafe { nrf5340_app_pac::Peripherals::steal().FICR_S }.info;

        let low = ficr.deviceid[0].read().bits();
        let high = ficr.deviceid[1].read().bits();
        Ok(Self((u64::from(high) << u32::BITS) | u64::from(low)))
    }

    type Bytes = [u8; 8];

    fn bytes(&self) -> Self::Bytes {
        self.0.to_le_bytes()
    }
}
