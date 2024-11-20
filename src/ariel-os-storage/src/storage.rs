//! Storage module wrapping [`sequential_storage`] in an object together with
//! a flash range and backend.
use core::ops::Range;

use arrayvec::ArrayString;
use embedded_storage_async::nor_flash::{ErrorType, MultiwriteNorFlash, NorFlash};
use sequential_storage::{
    cache::NoCache,
    erase_all,
    map::{fetch_item, remove_item, store_item, Value},
};

pub use crate::postcard_value::PostcardValue;
pub use serde::{Deserialize, Serialize};

/// Maximum key length.
pub const MAX_KEY_LEN: usize = 64usize;
/// Data buffer length.
pub const DATA_BUFFER_SIZE: usize = 128usize;

/// Object holding an instance of a key-value pair storage.
///
/// You should probably look into using the global instance accessible via
/// `ariel_os_storage::storage::{get,insert,remove}`.
pub struct Storage<F> {
    flash: F,
    storage_range: Range<u32>,
}

impl<F: NorFlash> Storage<F> {
    /// Creates a new [`Storage`] instance.
    pub const fn new(flash: F, storage_range: Range<u32>) -> Storage<F> {
        Self {
            flash,
            storage_range,
        }
    }

    /// Gets a [`Value`] from this [`Storage`] instance.
    ///
    /// # Panics
    ///
    /// Currently panics if `key.len() > MAX_KEY_LEN`.
    pub async fn get_raw<V: for<'d> Value<'d>>(
        &mut self,
        key: &str,
    ) -> Result<Option<V>, sequential_storage::Error<<F as ErrorType>::Error>> {
        let key = ArrayString::<MAX_KEY_LEN>::from(key).unwrap();
        let mut data_buffer = [0; DATA_BUFFER_SIZE];

        fetch_item::<_, V, _>(
            &mut self.flash,
            self.storage_range.clone(),
            &mut NoCache::new(),
            &mut data_buffer,
            &key,
        )
        .await
    }

    /// Inserts a [`Value`] into this [`Storage`] instance.
    ///
    /// # Panics
    ///
    /// Currently panics if `key.len() > MAX_KEY_LEN`.
    pub async fn insert_raw<'d, V: Value<'d>>(
        &mut self,
        key: &str,
        value: V,
    ) -> Result<(), sequential_storage::Error<<F as ErrorType>::Error>> {
        let key = ArrayString::<MAX_KEY_LEN>::from(key).unwrap();
        let mut data_buffer = [0; DATA_BUFFER_SIZE];
        store_item(
            &mut self.flash,
            self.storage_range.clone(),
            &mut NoCache::new(),
            &mut data_buffer,
            &key,
            &value,
        )
        .await
    }

    /// Stores a key-value pair into flash memory.
    ///
    /// It will overwrite the last value that has the same key.
    pub async fn insert<'d, V>(
        &mut self,
        key: &str,
        value: V,
    ) -> Result<(), sequential_storage::Error<<F as ErrorType>::Error>>
    where
        V: Serialize + Deserialize<'d> + Into<PostcardValue<V>>,
    {
        self.insert_raw(key, value.into()).await
    }

    /// Gets the last stored value from the flash that is associated with the given key.
    ///
    /// If no value with the key is found, `None` is returned.
    ///
    /// # Panics
    ///
    /// Currently panics if `key.len() > MAX_KEY_LEN`.
    pub async fn get<V>(
        &mut self,
        key: &str,
    ) -> Result<Option<V>, sequential_storage::Error<<F as ErrorType>::Error>>
    where
        V: Serialize + for<'d> Deserialize<'d> + Into<PostcardValue<V>>,
    {
        let key = ArrayString::<MAX_KEY_LEN>::from(key).unwrap();
        let mut data_buffer = [0; DATA_BUFFER_SIZE];

        let postcard_value = fetch_item::<_, PostcardValue<V>, _>(
            &mut self.flash,
            self.storage_range.clone(),
            &mut NoCache::new(),
            &mut data_buffer,
            &key,
        )
        .await?;
        Ok(postcard_value.map(PostcardValue::into_inner))
    }

    /// Resets the flash in the entire flash range of this [`Storage`] instance.
    pub async fn erase_all(
        &mut self,
    ) -> Result<(), sequential_storage::Error<<F as ErrorType>::Error>> {
        erase_all(&mut self.flash, self.storage_range.clone()).await
    }
}

impl<F: MultiwriteNorFlash> Storage<F> {
    /// Deletes an item from flash.
    ///
    /// Additional calls to [`Storage::get()`] with the same key will return `None` until
    /// a new one is stored again.
    ///
    /// <div class="warning">
    /// This is really slow!
    ///
    /// All items in flash have to be read and deserialized to find the items with the key.
    /// This is unlikely to be cached well.
    /// </div>
    ///
    /// # Panics
    ///
    /// Currently panics if `key.len() > MAX_KEY_LEN`.
    pub async fn remove(
        &mut self,
        key: &str,
    ) -> Result<(), sequential_storage::Error<<F as ErrorType>::Error>> {
        let key = ArrayString::<MAX_KEY_LEN>::from(key).unwrap();
        let mut data_buffer = [0; DATA_BUFFER_SIZE];
        remove_item(
            &mut self.flash,
            self.storage_range.clone(),
            &mut NoCache::new(),
            &mut data_buffer,
            &key,
        )
        .await
    }
}
