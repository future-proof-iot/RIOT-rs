#![cfg_attr(not(test), no_std)]

//! This module provides a FIFO index queue that can be used for implementing
//! a ring buffer.
//!
//! It keeps track of indexes from 0..N (with N being a power of two).
//!
//! `put()` marks an index "used".
//! `get()` returns an indexe that has been `put()` (if any) and marks it unused.
//! `peek()` returns the index that `get()` would return next
//! (if any) without marking it unused.
//!
//! All operations are O(1).

#[derive(Debug)]
pub struct RingBufferIndex {
    reads: u8,
    writes: u8,
    mask: u8,
}

const fn next_smaller_power_of_two(val: u8) -> u8 {
    if val.is_power_of_two() {
        val
    } else {
        ((val >> 1) + 1).next_power_of_two()
    }
}

impl RingBufferIndex {
    /// Create a new RingBufferIndex instance.
    ///
    /// `size` *should* be a power of two. Only `floor(log2(size))` elements will
    /// ever be used.
    pub const fn new(size: u8) -> RingBufferIndex {
        RingBufferIndex {
            reads: 0,
            writes: 0,
            mask: next_smaller_power_of_two(size) - 1,
        }
    }

    /// Returns the number of slots available for `get()`
    pub fn available(&self) -> u8 {
        self.writes - self.reads
    }

    /// Returns `true` if no element is available for `get()`
    pub fn is_empty(&self) -> bool {
        self.writes.wrapping_sub(self.reads) == 0
    }

    /// Returns `true` if no element can be `put()`
    pub fn is_full(&self) -> bool {
        // sadly the first check is necessary to not break
        // for zero-sized indexes
        (self.mask == 0) || (self.available() > self.mask)
    }

    /// Returns a "used" index (if any) and marks it unused.
    pub fn get(&mut self) -> Option<u8> {
        if !self.is_empty() {
            let reads = self.reads;
            self.reads = reads.wrapping_add(1);
            Some(reads & self.mask)
        } else {
            None
        }
    }

    /// Returns a "used" index (if any).
    pub fn peek(&self) -> Option<u8> {
        if !self.is_empty() {
            Some(self.reads & self.mask)
        } else {
            None
        }
    }

    /// Marks the next available index "used" (if any) and returns it.
    pub fn put(&mut self) -> Option<u8> {
        if !self.is_full() {
            let writes = self.writes;
            self.writes = writes.wrapping_add(1);
            Some(writes & self.mask)
        } else {
            None
        }
    }

    /// Returns the total capacity of indexes that this instance keeps track of.
    pub fn capacity(&self) -> usize {
        if self.mask > 0 {
            self.mask as usize + 1
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn basic() {
        let mut rb = super::RingBufferIndex::new(4);
        assert!(rb.is_empty());
        assert_eq!(rb.available(), 0);
        assert!(!rb.is_full());
        assert_eq!(rb.get(), None);

        assert_eq!(rb.put(), Some(0u8));
        assert_eq!(rb.available(), 1);
        assert_eq!(rb.put(), Some(1u8));
        assert_eq!(rb.available(), 2);
        assert_eq!(rb.put(), Some(2u8));
        assert_eq!(rb.available(), 3);
        assert_eq!(rb.put(), Some(3u8));
        assert_eq!(rb.available(), 4);

        assert_eq!(rb.put(), None);
        assert!(rb.is_full());
        assert!(!rb.is_empty());

        assert_eq!(rb.get(), Some(0u8));
        assert_eq!(rb.available(), 3);
        assert_eq!(rb.get(), Some(1u8));
        assert_eq!(rb.available(), 2);
        assert_eq!(rb.get(), Some(2u8));
        assert_eq!(rb.available(), 1);
        assert_eq!(rb.get(), Some(3u8));
        assert_eq!(rb.available(), 0);

        assert!(rb.is_empty());
        assert_eq!(rb.get(), None);
    }

    #[cfg(test)]
    fn test_with_size(size: u8, n: usize) {
        let mut rb = super::RingBufferIndex::new(size);
        for i in 0..n {
            assert_eq!(
                rb.put(),
                Some(i as u8 % super::next_smaller_power_of_two(size))
            );
            assert_eq!(
                rb.get(),
                Some(i as u8 % super::next_smaller_power_of_two(size))
            );
        }
    }

    #[test]
    fn counter_overflow_size4() {
        test_with_size(4, 16);
    }

    #[test]
    fn counter_overflow_size128() {
        test_with_size(128, 256);
    }

    #[test]
    fn counter_overflow_size255() {
        test_with_size(255, 1024);
    }

    #[test]
    fn test_is_full() {
        let size = 128;
        let mut rb = super::RingBufferIndex::new(size);
        for i in 0..size {
            assert_eq!(rb.put(), Some(i as u8));
        }
        assert_eq!(rb.available(), 128);
        assert!(rb.is_full());
    }

    #[test]
    fn test_next_smaller_power_of_two() {
        assert_eq!(super::next_smaller_power_of_two(0), 1);
        assert_eq!(super::next_smaller_power_of_two(1), 1);
        assert_eq!(super::next_smaller_power_of_two(2), 2);
        assert_eq!(super::next_smaller_power_of_two(3), 2);
        assert_eq!(super::next_smaller_power_of_two(4), 4);
        assert_eq!(super::next_smaller_power_of_two(5), 4);
        assert_eq!(super::next_smaller_power_of_two(63), 32);
        assert_eq!(super::next_smaller_power_of_two(128), 128);
        assert_eq!(super::next_smaller_power_of_two(129), 128);
        assert_eq!(super::next_smaller_power_of_two(255), 128);
    }

    #[test]
    fn zero_sized() {
        let mut rb = super::RingBufferIndex::new(0);
        println!("{:#?}", &rb);
        assert!(rb.is_full());
        assert!(rb.is_empty());
        assert_eq!(rb.put(), None);
    }
}
