#[esp_hal::entry]
fn main() -> ! {
    crate::startup();
}

pub fn init() {}
