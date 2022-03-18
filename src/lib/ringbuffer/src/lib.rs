#![cfg_attr(not(test), no_std)]
#![feature(const_mut_refs)]

//! Typed FIFO ringbuffer supporting single element put/get/peek
//!
//! This implementation allows to be initialized without backing storage.

use core::mem::MaybeUninit;
use rbi::RingBufferIndex;

#[derive(Debug)]
pub struct RingBuffer<'a, T>
where
    T: Copy + Sized,
{
    index: RingBufferIndex,
    slice: Option<&'a mut [T]>,
}

impl<'a, T> RingBuffer<'a, T>
where
    T: Copy + Sized,
{
    pub const fn new(backing_array: Option<&mut [MaybeUninit<T>]>) -> RingBuffer<T> {
        if let Some(backing_array) = backing_array {
            RingBuffer {
                index: RingBufferIndex::new(backing_array.len() as u8),
                // this is basically MaybeUninit::slice_assume_init_mut().
                // Cannot use that as it is not const.
                slice: Some(unsafe { &mut *(backing_array as *mut [MaybeUninit<T>] as *mut [T]) }),
            }
        } else {
            RingBuffer {
                index: RingBufferIndex::new(0),
                slice: None,
            }
        }
    }

    pub fn put(&mut self, element: T) -> bool {
        if let Some(pos) = self.index.put() {
            self.slice.as_mut().unwrap()[pos as usize] = element;
            true
        } else {
            false
        }
    }

    pub fn get(&mut self) -> Option<T> {
        if let Some(pos) = self.index.get() {
            // safety: this only returns elements that have been
            // stored with put()
            Some(self.slice.as_ref().unwrap()[pos as usize])
        } else {
            None
        }
    }

    pub fn peek(&self) -> Option<T> {
        if let Some(pos) = self.index.peek() {
            // safety: this only returns elements that have been
            // stored with put()
            Some(self.slice.as_ref().unwrap()[pos as usize])
        } else {
            None
        }
    }

    pub fn available(&self) -> usize {
        self.index.available() as usize
    }

    pub fn capacity(&self) -> usize {
        self.index.capacity()
    }

    pub fn is_full(&self) -> bool {
        self.index.is_full()
    }

    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    pub fn set_backing_array(&mut self, array: Option<&'a mut [T]>) {
        let len = if let Some(array) = &array {
            array.len()
        } else {
            0
        };
        self.slice = array;
        self.index = RingBufferIndex::new(len as u8);
    }
}

#[cfg(test)]
mod tests {
    use super::RingBuffer;
    use core::mem::MaybeUninit;
    #[test]
    fn basic() {
        let mut array: [MaybeUninit<char>; 16] = unsafe { MaybeUninit::uninit().assume_init() };
        let mut rb = RingBuffer::new(Some(&mut array));
        assert!(rb.put('0'));
        assert_eq!(rb.peek(), Some('0'));
        assert_eq!(rb.get(), Some('0'));
    }
}
