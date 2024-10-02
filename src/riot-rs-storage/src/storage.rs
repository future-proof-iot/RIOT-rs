use core::ops::Range;

use arrayvec::ArrayString;
pub use embedded_storage_async::nor_flash::ErrorType;
use embedded_storage_async::nor_flash::{MultiwriteNorFlash, NorFlash};
pub use serde::{Deserialize, Serialize};

use sequential_storage::{
    cache::NoCache,
    map::{fetch_item, remove_item, store_item, SerializationError, Value},
};

pub const MAX_KEY_LEN: usize = 64usize;
pub const DATA_BUFFER_SIZE: usize = 128usize;

pub struct Storage<F> {
    flash: F,
    storage_range: Range<u32>,
}

pub struct StringValue {
    pub inner: ArrayString<MAX_KEY_LEN>,
}

impl StringValue {
    pub fn from(string: &str) -> Self {
        Self {
            inner: ArrayString::<MAX_KEY_LEN>::from(string).unwrap(),
        }
    }
}

impl<'d> Value<'d> for StringValue {
    fn serialize_into(
        &self,
        buffer: &mut [u8],
    ) -> Result<usize, sequential_storage::map::SerializationError> {
        buffer[0..self.inner.len()].copy_from_slice(self.inner.as_bytes());
        Ok(self.inner.len())
    }
    fn deserialize_from(
        buffer: &'d [u8],
    ) -> Result<Self, sequential_storage::map::SerializationError> {
        let mut output = ArrayString::<MAX_KEY_LEN>::new();
        output
            .try_push_str(
                core::str::from_utf8(buffer).map_err(|_| SerializationError::InvalidFormat)?,
            )
            .map_err(|_| SerializationError::InvalidFormat)?;

        Ok(Self { inner: output })
    }
}

pub use crate::postcard_value::PostcardValue;

impl<F: NorFlash> Storage<F> {
    pub const fn new(flash: F, storage_range: Range<u32>) -> Storage<F> {
        Self {
            flash,
            storage_range,
        }
    }

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

    pub async fn put_raw<'d, V: Value<'d>>(
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

    pub async fn put<'d, V>(
        &mut self,
        key: &str,
        value: V,
    ) -> Result<(), sequential_storage::Error<<F as ErrorType>::Error>>
    where
        V: Serialize + Deserialize<'d> + Into<PostcardValue<V>>,
    {
        self.put_raw(key, value.into()).await
    }

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
        Ok(postcard_value.map(|v| v.into_inner()))
    }
}

impl<F: MultiwriteNorFlash> Storage<F> {
    pub async fn del(
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
