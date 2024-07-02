//! An owned heapless container with capacity N that maintains order both through properties of its
//! entries and by time of access -- a mix between a priority queue and an LRU cache.
//!
//! This was inspired by the [`uluru`](https://github.com/servo/uluru) crate. Adapting that crate
//! to support priority levels proved to be impractical -- and many properties have changed since.
//!
//! See the [`OrderedPool`] documentation for details.
//!
//! # Terminology
//!
//! The ordering imposed on the cache entries is determined by a priority (see [`PriorityLevel`]).
//! Throughout the documentation, "high" and "low" indicate priorities, whereas "small" and "large"
//! indicate numeric values (including of priorities, where "high" corresponds to "small" and "low"
//! to "large").
#![forbid(unsafe_code)]

use arrayvec::ArrayVec;

/// Required trait on [`OrderedPool`] entries that allows ordering.
///
/// Priority levels follow the conventions common with schedulers: 0 is the highest priority, and
/// will only get evicted if the cache is full with other entries of the same priority. Larger
/// numeric values indicate increasingly lower priority.
pub trait PriorityLevel {
    /// Calculate the priority of the instance
    ///
    /// An instance's priority level may change while being mutated; [`OrderedPool`] will account for
    /// that.
    ///
    /// The level should not change due to global effects (or internal mutability, if shared access
    /// is later implemented). If it does, the ordering of an [`OrderedPool`] containing it may become
    /// arbitrary, even after the element whose level changed has been removed.
    fn level(&self) -> usize;
}

/// An owned heapless container with capacity `N` that maintains order both through properties of its
/// entries and by time of access.
///
/// Operations that are supposed to be fast are:
///
/// * Finding an element by iteration
/// * Moving that element to the front of its relevant level
///
/// There is no remove an item; instead, the `&mut T` of [`Self::lookup`] can be replaced with a
/// low-priority placeholder value. (In fact, future iterations may require that such a value
/// exists and is [`Default`]).
///
/// # Usage
///
/// ```ignore
/// use crate::oluru::{OrderedPool, PriorityLevel};
///
/// #[derive(Debug)]
/// struct MyValue {
///     id: u32,
///     name: Option<&'static str>,
/// }
///
/// impl PriorityLevel for MyValue {
///     fn level(&self) -> usize {
///         if self.name.is_some() {
///             0
///         } else {
///             1
///         }
///     }
/// }
///
/// // A cache with a capacity of three.
/// type MyCache = OrderedPool<MyValue, 3, 2>;
///
/// // Create an empty cache, then insert some items.
/// let mut cache = MyCache::new();
/// cache.insert(MyValue { id: 1, name: Some("Mercury") });
/// cache.insert(MyValue { id: 2, name: Some("Venus") });
/// cache.insert(MyValue { id: 3, name: None });
///
/// let item = cache.lookup(|x| x.id == 1, |x| format!("Found {}", x.name.unwrap_or("unnamed object")));
/// assert_eq!(item.unwrap().as_str(), "Found Mercury");
///
/// // If the cache is full, inserting a new item evicts one item.
/// //
/// // While Venus (ID 2) was used least recently, it has higher priority than the no-name planet
/// // with index 3, so that gets evicted first instead.
/// let returned = cache.insert(MyValue { id: 4, name: Some("Mars") });
/// assert!(returned.expect("Pool was full").is_some_and(|item| item.id == 3));
/// ```
///
/// # Implementation
///
/// Right now, this is implemented as a separate entry vector and an index vector, where the latter
/// is often rotated internally. Future changes may change this to only be a single list, using a
/// doubly linked list, and keeping head indices of each level (which is why the number of levels
/// `L` is part of the type).
///
/// The value list following the style of a `Vec` means that popping elements from anywhere but the
/// tail is costly, which it should better not be; a slab allocator style would improve that.
///
/// ## Terminology
///
/// A "position" is a key to `.sorted`. An "index" is a key to `.entries` (and thus a value of
/// `.sorted`).
///
/// ## Invariants
///
/// Before and after any public function, these hold:
///
/// * `.sorted` has the same length as `.entries`
/// * `.sorted` is a permutation of `.entries`' (and thus its) index space. Therefore, each of its
///   values is unique, and is an index into `.entries`.
/// * If `T::level` is constant, `self.sorted.iter().map(|i| self.entries[i].level())` is sorted.
#[derive(Debug)]
pub struct OrderedPool<T: PriorityLevel, const N: usize, const L: usize> {
    /// Elements without regard for ordering
    pub entries: ArrayVec<T, N>,
    /// A sorted list of indices into entries: high priority first, ties broken by recentness
    pub sorted: ArrayVec<u16, N>,
}

impl<T: PriorityLevel, const N: usize, const L: usize> OrderedPool<T, N, L> {
    /// Create an empty cache.
    pub const fn new() -> Self {
        assert!(N < u16::MAX as usize, "Capacity overflow");
        // Clipping levels to u16 because they may be stored if the implementation changes.
        assert!(L < u16::MAX as usize, "Level overflow");
        OrderedPool {
            entries: ArrayVec::new_const(),
            sorted: ArrayVec::new_const(),
        }
    }

    /// Iterate over items from highest priority to lowest, most recently used first.
    ///
    /// If the function `f_test` (which receives a shared reference to the entry) returns
    /// `Some(r)`, `f_use` is called with a *mutable* reference to the item as well as the first
    /// function's result. That item is regarded as "used" and thus shifted to the front, and the
    /// second function's return value is passed on.
    ///
    /// This differs from `uluru` in two ways:
    /// * There is no `find()` method: As the level may change through mutation, we can not hand
    ///   out a `&mut T` unless we can sure to process any level changes when it is returned. (A
    ///   linear type drop guard wrapper may afford that, but is not available in Rust at the time
    ///   of writing)
    /// * The callback is split in a test part and a use part, which ensures that elements that are
    ///   not looked up do not get mutated; only the selected item is mutated and will then be
    ///   sorted in correctly.
    pub fn lookup<Ftest, Fuse, R>(&mut self, mut f_test: Ftest, f_use: Fuse) -> Option<R>
    where
        Ftest: FnMut(&T) -> bool,
        Fuse: FnOnce(&mut T) -> R,
    {
        for (position, &index) in self.sorted.iter().enumerate() {
            if f_test(&self.entries[usize::from(index)]) {
                let r = f_use(&mut self.entries[usize::from(index)]);
                self.touch(position);
                return Some(r);
            }
        }
        None
    }

    /// Inserts an element.
    ///
    /// If the new element's priority is lower than the lowest in the queue, it is returned as an
    /// Err. Otherwise, the element is inserted, and any dropped lower priority element is
    /// returned in the Ok value.
    pub fn insert(&mut self, new: T) -> Result<Option<T>, T> {
        let new_index = self.entries.len();
        if new_index < N {
            self.entries.push(new);
            self.sorted.push(
                new_index
                    .try_into()
                    .expect("Range is checked at construction time"),
            );
            self.touch(new_index);
            Ok(None)
        } else {
            let last_slot = &mut self.entries
                [usize::from(*self.sorted.last().expect("Full array is not empty"))];
            let last_level = last_slot.level();
            let new_level = new.level();
            debug_assert!(new_level < L, "Level exceeds limit L={L} in type");
            if new_level <= last_level {
                let last = core::mem::replace(last_slot, new);
                self.touch(N - 1);
                Ok(Some(last))
            } else {
                Err(new)
            }
        }
    }

    // FIXME: It is not fully clear when we would use insert and force_insert -- we'll need some
    // kind of force_insert to get out of situations where all authenticated connections are really
    // timing out. But at the same time, if we're being bombarded with bad requests, we should
    // retain the ability to randomly keep some incoming connection for longer, so that a message 3
    // that comes through can establish the connection. In the end, we may need some more flexible
    // policy than just levels and LRU.
    //
    // Unless we want to keep track of connections somewhere in parallel, timeouts may also involve
    // some function called on all present items to mark them as "not used in a long time",
    // downgrading their priority.

    /// Inserts an element without regard for its level.
    ///
    /// The element is inserted unconditionally, and the least priority element is returned by
    /// value.
    pub fn force_insert(&mut self, new: T) -> Option<T> {
        let new_index = self.entries.len();
        if new_index < N {
            self.entries.push(new);
            self.sorted.push(
                new_index
                    .try_into()
                    .expect("Range is checked at construction time"),
            );
            self.touch(new_index);
            None
        } else {
            let last_slot = &mut self.entries
                [usize::from(*self.sorted.last().expect("Full array is not empty"))];
            let last = core::mem::replace(last_slot, new);
            self.touch(N - 1);
            Some(last)
        }
    }

    fn touch(&mut self, position: usize) {
        let level = self.entries[usize::from(self.sorted[position])].level();
        debug_assert!(level < L, "Level exceeds limit L={L} in type");
        let mut new_position = position;
        // Common case: level stayed the same, but we move to front; also applicable when numeric
        // level decrased
        while new_position
            .checked_sub(1)
            .is_some_and(|n| self.entries[usize::from(self.sorted[n])].level() >= level)
        {
            new_position -= 1;
        }
        if new_position != position {
            // Push our entry out right and in left in the front
            self.sorted[new_position..=position].rotate_right(1);
        } else {
            // Level may instead have increased
            while new_position < self.sorted.len() - 1
                && self.entries[usize::from(self.sorted[new_position + 1])].level() < level
            {
                new_position += 1;
            }
            // Push our entry out left and in right in the rear
            if new_position != position {
                self.sorted[position..=new_position].rotate_left(1);
            }
        }
    }

    /// Returns an iterator visiting all items in arbitrary order.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.entries.iter()
    }
}

impl<T: PriorityLevel, const N: usize, const L: usize> core::default::Default
    for OrderedPool<T, N, L>
{
    fn default() -> Self {
        Self::new()
    }
}
