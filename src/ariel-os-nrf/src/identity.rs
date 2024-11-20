pub struct DeviceId(u64);

impl ariel_os_embassy_common::identity::DeviceId for DeviceId {
    #[expect(
        refining_impl_trait_reachable,
        reason = "making this fallible would be a breaking API change for Ariel OS"
    )]
    fn get() -> Result<Self, core::convert::Infallible> {
        #[cfg(not(context = "nrf5340"))]
        let ficr = embassy_nrf::pac::FICR;
        #[cfg(context = "nrf5340")]
        let ficr = embassy_nrf::pac::FICR.info();

        let low = ficr.deviceid(0).read();
        let high = ficr.deviceid(1).read();
        Ok(Self((u64::from(high) << u32::BITS) | u64::from(low)))
    }

    type Bytes = [u8; 8];

    fn bytes(&self) -> Self::Bytes {
        self.0.to_le_bytes()
    }
}
