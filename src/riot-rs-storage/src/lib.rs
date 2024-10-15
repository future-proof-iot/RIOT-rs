#![cfg_attr(not(test), no_std)]

use core::ops::Range;
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    mutex::{Mutex, MutexGuard},
    once_lock::OnceLock,
};

cfg_if::cfg_if! {
    if #[cfg(context = "nrf")] {
        pub use riot_rs_nrf as arch;
    } else if #[cfg(context = "rp")] {
        pub use riot_rs_rp as arch;
    } else if #[cfg(context = "esp")] {
        pub use riot_rs_esp as arch;
    } else if #[cfg(context = "stm32")] {
        pub use riot_rs_stm32 as arch;
    } else if #[cfg(context = "riot-rs")] {
        compile_error!("this architecture is not supported");
    } else {
        pub mod arch;
    }
}

use arch::storage::{init as flash_init, Flash, FlashError};
use arch::OptionalPeripherals;

mod postcard_value;
pub mod storage;

pub use storage::*;

static STORAGE: OnceLock<Mutex<CriticalSectionRawMutex, Storage<Flash>>> = OnceLock::new();

fn flash_range_from_linker() -> Range<u32> {
    #[cfg(context = "rp2040")]
    const OFFSET: usize = 0x10000000;
    #[cfg(not(context = "rp2040"))]
    const OFFSET: usize = 0x0;

    extern "C" {
        static __storage_start: u32;
        static __storage_end: u32;
    }

    let start = core::ptr::addr_of!(__storage_start) as usize - OFFSET;
    let end = core::ptr::addr_of!(__storage_end) as usize - OFFSET;

    start as u32..end as u32
}

pub fn init(p: &mut OptionalPeripherals) {
    use riot_rs_debug::log::info;
    let flash_range = flash_range_from_linker();
    info!("storage: using flash range {}", flash_range);

    let flash = flash_init(p);
    let _ = STORAGE.init(Mutex::new(Storage::new(flash, flash_range)));
}

pub async fn get_mutex() -> MutexGuard<'static, CriticalSectionRawMutex, storage::Storage<Flash>> {
    STORAGE.get().await.lock().await
}

pub async fn put<'d, V>(key: &str, value: V) -> Result<(), sequential_storage::Error<FlashError>>
where
    V: Serialize + Deserialize<'d> + Into<PostcardValue<V>>,
{
    get_mutex().await.put::<V>(key, value).await
}

pub async fn get<V>(key: &str) -> Result<Option<V>, sequential_storage::Error<FlashError>>
where
    V: Serialize + for<'d> Deserialize<'d> + Into<PostcardValue<V>>,
{
    get_mutex().await.get(key).await
}

pub async fn del(key: &str) -> Result<(), sequential_storage::Error<FlashError>> {
    get_mutex().await.del(key).await
}
