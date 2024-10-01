#[derive(Debug, defmt::Format)]
pub struct DeviceId(&'static [u8; 12]);

impl riot_rs_embassy_common::identity::DeviceId for DeviceId {
    type Bytes = &'static [u8; 12];

    type Error = core::convert::Infallible;

    fn get() -> Result<Self, Self::Error> {
        Ok(Self(embassy_stm32::uid::uid()))
    }

    fn bytes(&self) -> Self::Bytes {
        self.0
    }
}
