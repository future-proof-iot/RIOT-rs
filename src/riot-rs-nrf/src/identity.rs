pub struct DeviceId;

impl riot_rs_embassy_common::identity::DeviceId for DeviceId {
    /// The DEVICEID from the FICR peripheral.
    ///
    /// The two-word value is interpreted as a 64-bit value as per the product specification.
    type DeviceId = u64;

    type Error = core::convert::Infallible;

    fn get() -> Result<Self::DeviceId, Self::Error> {
        // Embassy does not wrap the FICR register, and given that all we need from there is a register
        // read that is perfectly fine to do through a stolen register, let's do that rather than
        // thread the access through several layers.

        // SAFETY: The register is used for read-only operations on constant values.
        #[cfg(context = "nrf52840")]
        let ficr = unsafe { nrf52840_pac::Peripherals::steal().FICR };
        #[cfg(context = "nrf52832")]
        let ficr = unsafe { nrf52832_pac::Peripherals::steal().FICR };
        #[cfg(context = "nrf5340")]
        let ficr = unsafe { nrf5340_app_pac::Peripherals::steal().FICR_S }.info;

        let low = ficr.deviceid[0].read().bits();
        let high = ficr.deviceid[1].read().bits();
        Ok((u64::from(high) << u32::BITS) | u64::from(low))
    }
}
