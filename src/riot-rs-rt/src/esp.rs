#[esp_hal::entry]
fn main() -> ! {
    crate::startup();
}

pub fn init() {}

pub fn benchmark<F: Fn() -> ()>(_iterations: usize, _f: F) -> core::result::Result<usize, ()> {
    unimplemented!()
}
