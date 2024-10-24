use embassy_rp::{flash::Async, peripherals::FLASH};

pub type Flash = embassy_rp::flash::Flash<'static, FLASH, Async, FLASH_SIZE>;
pub type FlashError = embassy_rp::flash::Error;

const FLASH_SIZE: usize = 2 * 1024 * 1024;

pub fn init(p: &mut crate::OptionalPeripherals) -> Flash {
    embassy_rp::flash::Flash::<_, Async, FLASH_SIZE>::new(
        p.FLASH.take().unwrap(),
        p.DMA_CH0.take().unwrap(),
    )
}
