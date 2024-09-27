pub struct DeviceId;

impl riot_rs_embassy_common::identity::DeviceId for DeviceId {
    type DeviceId = &'static [u8; 12];

    type Error = core::convert::Infallible;

    fn get() -> Result<Self::DeviceId, Self::Error> {
        Ok(embassy_stm32::uid::uid())
    }
}
