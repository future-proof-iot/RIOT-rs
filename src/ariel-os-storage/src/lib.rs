//! Provides key-value pair persistent storage on flash.
//!
//! Currently the same type used for serializing must be used for deserializing.
//! While not doing so won't cause unsafety, it might return garbage data, or panic.

#![cfg_attr(not(test), no_std)]
#![deny(missing_docs)]
#![deny(clippy::pedantic)]
// TODO: overhaul errors
#![expect(clippy::missing_errors_doc)]

mod postcard_value;
mod storage;

use core::ops::Range;

use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    mutex::{Mutex, MutexGuard},
    once_lock::OnceLock,
};
use riot_rs_hal::{
    storage::{init as flash_init, Flash, FlashError},
    OptionalPeripherals,
};

pub use storage::*;

static STORAGE: OnceLock<Mutex<CriticalSectionRawMutex, Storage<Flash>>> = OnceLock::new();

/// Gets a [`Range`] from the linker that can be used for a global [`Storage`].
///
/// This expects two symbols `__storage_start` and `__storage_end`.
/// This function is also the place to configure a platform dependent `OFFSET`,
/// which configures an offset between the linker flash address map and the
/// flash driver address map.
fn flash_range_from_linker() -> Range<u32> {
    #[cfg(context = "rp2040")]
    const OFFSET: usize = 0x10000000;
    #[cfg(not(context = "rp2040"))]
    const OFFSET: usize = 0x0;

    extern "C" {
        static __storage_start: u32;
        static __storage_end: u32;
    }

    let start = &raw const __storage_start as usize - OFFSET;
    let end = &raw const __storage_end as usize - OFFSET;

    #[expect(clippy::cast_possible_truncation)]
    let (start, end) = (start as u32, end as u32);

    start..end
}

fn init_(p: &mut OptionalPeripherals) {
    use riot_rs_debug::log::info;
    let flash_range = flash_range_from_linker();
    info!("storage: using flash range {}", &flash_range);

    let flash = flash_init(p);
    let _ = STORAGE.init(Mutex::new(Storage::new(flash, flash_range)));
}

/// Initializes the global storage.
///
/// Note: this is automatically called by the Ariel OS initialization code.
///
/// # Panics
///
/// Panics when initializing the flash fails.
#[doc(hidden)]
pub async fn init(p: &mut OptionalPeripherals) {
    const MARKER_KEY: &str = "0xdeadcafe";
    const MARKER_VALUE: u32 = 0xdead_cafe;

    init_(p);

    // add some delay to give an attached debug probe time to parse the
    // defmt RTT header. Reading that header might touch flash memory, which
    // interferes with flash write operations.
    // https://github.com/knurling-rs/defmt/pull/683
    #[cfg(context = "rp")]
    embassy_time::Timer::after_millis(10).await;

    // Use a marker to ensure that this storage is initialized.
    match get::<u32>(MARKER_KEY).await {
        Ok(Some(val)) if val == MARKER_VALUE => {
            // all good
        }
        _ => {
            riot_rs_debug::log::info!("storage: initializing");
            let mut s = lock().await;
            s.erase_all().await.unwrap();
            s.insert(MARKER_KEY, MARKER_VALUE).await.unwrap();
        }
    }
}

/// Stores a key-value pair into flash memory.
///
/// It will overwrite the last value that has the same key.
pub async fn insert<'d, V>(key: &str, value: V) -> Result<(), sequential_storage::Error<FlashError>>
where
    V: Serialize + Deserialize<'d> + Into<PostcardValue<V>>,
{
    lock().await.insert::<V>(key, value).await
}

/// Gets the last stored value from the flash that is associated with the given key.
///
/// Note: Always [`get()`] the same value type that was [`insert()`]!
///
/// If no value with the key is found, `None` is returned.
pub async fn get<V>(key: &str) -> Result<Option<V>, sequential_storage::Error<FlashError>>
where
    V: Serialize + for<'d> Deserialize<'d> + Into<PostcardValue<V>>,
{
    lock().await.get(key).await
}

/// Deletes an item from flash.
///
/// Additional calls to [`get()`] with the same key will return `None` until
/// a new one is stored again.
///
/// <div class="warning">
/// This is really slow!
///
/// All items in flash have to be read and deserialized to find the items with the key.
/// This is unlikely to be cached well.
/// </div>
pub async fn remove(key: &str) -> Result<(), sequential_storage::Error<FlashError>> {
    lock().await.remove(key).await
}

/// Gets a [`MutexGuard`] of the global [`Storage`] object.
///
/// This can be used to implement atomic RMW (like counters).
/// *It is not needed for using the global [`get()`], [`insert()`] and [`remove()`] functions.*
///
/// Note: don't forget to drop the mutex guard returned by this.
///
/// Example:
///
/// ```ignore
/// {
///     // By getting the Storage mutex directly, changing e.g., a counter,
///     // can be done atomically w.r.t. concurrent access from the same firmware:
///     let mut s = storage::lock().await;
///     let value: Option<u32> = s.get("counter").await.unwrap();
///     let value = value.unwrap_or_default();
///     s.insert("counter", value + 1).await.unwrap();
/// }
/// ```
pub async fn lock() -> MutexGuard<'static, CriticalSectionRawMutex, storage::Storage<Flash>> {
    STORAGE.get().await.lock().await
}
