use embassy_embedded_hal::adapter::BlockingAsync;
use embassy_nrf::nvmc::Nvmc;

pub type Flash = BlockingAsync<Nvmc<'static>>;
pub type FlashError = embassy_nrf::nvmc::Error;

pub fn init(peripherals: &mut crate::OptionalPeripherals) -> Flash {
    let flash = Nvmc::new(peripherals.NVMC.take().unwrap());
    BlockingAsync::new(flash)
}
