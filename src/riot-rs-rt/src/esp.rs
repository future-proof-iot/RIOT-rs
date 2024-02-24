#[esp_hal::entry]
fn main() -> ! {
    super::startup();
}

pub fn init() {}

pub fn benchmark<F: Fn() -> ()>(iterations: usize, f: F) -> core::result::Result<usize, ()> {
    unimplemented!();
}
