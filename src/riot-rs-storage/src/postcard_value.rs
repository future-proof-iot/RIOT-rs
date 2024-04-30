use core::ops::Deref;

use postcard::{from_bytes, to_slice};
use serde::{Deserialize, Serialize};

use sequential_storage::map::{SerializationError, Value};

#[derive(Debug)]
pub struct PostcardValue<T> {
    value: T,
}

impl<'d, T: Serialize + Deserialize<'d>> PostcardValue<T> {
    pub const fn from(value: T) -> Self {
        Self { value }
    }
    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<'d, T: Serialize + Deserialize<'d>> From<T> for PostcardValue<T> {
    fn from(other: T) -> PostcardValue<T> {
        PostcardValue::from(other)
    }
}

impl<T> Deref for PostcardValue<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<'d, T: Serialize + Deserialize<'d>> Value<'d> for PostcardValue<T> {
    fn serialize_into(&self, buffer: &mut [u8]) -> Result<usize, SerializationError> {
        let used = to_slice(&self.value, buffer).map_err(|e| match e {
            postcard::Error::SerializeBufferFull => SerializationError::BufferTooSmall,
            _ => SerializationError::Custom(0),
        })?;

        Ok(used.len())
    }

    fn deserialize_from(buffer: &'d [u8]) -> Result<Self, SerializationError> {
        let value = from_bytes(buffer).map_err(|e| match e {
            postcard::Error::DeserializeUnexpectedEnd => SerializationError::InvalidData,
            _ => SerializationError::Custom(0),
        })?;

        Ok(Self { value })
    }
}
