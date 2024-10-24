#![allow(missing_docs)]
pub struct Flash;
impl embedded_storage_async::nor_flash::NorFlash for Flash {
    const ERASE_SIZE: usize = 42;
    const WRITE_SIZE: usize = 42;
    async fn write(
        &mut self,
        _: u32,
        _: &[u8],
    ) -> core::result::Result<(), <Self as embedded_storage_async::nor_flash::ErrorType>::Error>
    {
        todo!()
    }
    async fn erase(
        &mut self,
        _: u32,
        _: u32,
    ) -> core::result::Result<(), <Self as embedded_storage_async::nor_flash::ErrorType>::Error>
    {
        todo!()
    }
}
impl embedded_storage_async::nor_flash::ErrorType for Flash {
    type Error = FlashError;
}
impl embedded_storage_async::nor_flash::MultiwriteNorFlash for Flash {}
impl embedded_storage_async::nor_flash::ReadNorFlash for Flash {
    const READ_SIZE: usize = 42;
    async fn read(
        &mut self,
        _: u32,
        _: &mut [u8],
    ) -> core::result::Result<(), <Self as embedded_storage_async::nor_flash::ErrorType>::Error>
    {
        todo!()
    }
    fn capacity(&self) -> usize {
        todo!()
    }
}
#[derive(Debug)]
pub struct FlashError;
impl embedded_storage_async::nor_flash::NorFlashError for FlashError {
    fn kind(&self) -> embedded_storage_async::nor_flash::NorFlashErrorKind {
        todo!()
    }
}

#[must_use]
pub fn init(_: &mut crate::OptionalPeripherals) -> Flash {
    Flash {}
}
