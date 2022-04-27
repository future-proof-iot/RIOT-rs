//! This crate contains a circularly and singly linked list implementation.
//!
//! Its operations are:
//!
//! operation             | runtime | description
//! ----------------------|---------|---------------
//! clist::lpush()        | O(1)    | insert as head (leftmost node)
//! clist::lpeek()        | O(1)    | get the head without removing it
//! clist::lpop()         | O(1)    | remove and return head (leftmost node)
//! clist::rpush()        | O(1)    | append as tail (rightmost node)
//! clist::rpeek()        | O(1)    | get the tail without removing it
//! clist::rpop()         | O(n)    | remove and return tail (rightmost node)
//! clist::lpoprpush()    | O(1)    | move first element to the end of the list
//! clist::contains(      | O(n)    | check if list contains element
//! clist::find()         | O(n)    | find and return node
//! clist::find_before()  | O(n)    | find node return node pointing to node
//! clist::remove()       | O(n)    | remove and return node
//! clist::sort()         | O(NlogN)| sort list (stable)
//! clist::count()        | O(n)    | count the number of elements in a list
//!
//! clist can be used as a traditional list, a queue (FIFO) and a stack (LIFO) using
//! fast O(1) operations.
//!

#![cfg_attr(not(test), no_std)]
// features needed by our use of memoffset
#![feature(const_ptr_offset_from)]
#![feature(const_refs_to_cell)]

use core::cell::UnsafeCell;
use core::marker::PhantomPinned;

extern crate memoffset;
pub use memoffset::offset_of;

#[derive(Debug)]
pub struct Link {
    next: UnsafeCell<*const Link>,
    _pin: PhantomPinned,
}

pub struct List {
    last: Option<Link>,
}

unsafe impl Sync for List {}
unsafe impl Send for List {}
unsafe impl Sync for Link {}
unsafe impl Send for Link {}

impl Link {
    pub const fn new() -> Link {
        Link {
            next: UnsafeCell::new(core::ptr::null()),
            _pin: PhantomPinned,
        }
    }

    pub const unsafe fn new_linked(link: *const Link) -> Link {
        Link {
            next: UnsafeCell::new(link),
            _pin: PhantomPinned,
        }
    }

    /// check if this Link is currently part of a list.
    pub fn is_linked(&self) -> bool {
        self.next.get() == core::ptr::null_mut()
    }

    unsafe fn link(&self, next: &Link) {
        *self.next.get() = next as *const Link;
    }

    unsafe fn next_ptr(&self) -> *const Link {
        *self.next.get()
    }

    unsafe fn next(&self) -> &Link {
        &*self.next_ptr()
    }
}

// public
impl List {
    /// creates a new, empty list
    pub const fn new() -> Self {
        List { last: None }
    }

    /// returns true if list does not contain any elements
    pub fn is_empty(&self) -> bool {
        self.last.is_none()
    }

    /// Inserts element at the beginning of the list
    /// Complexity: O(1)
    pub fn lpush(&mut self, element: &mut Link) {
        if self.is_empty() {
            unsafe { self.push_initial_element(element) };
        } else {
            unsafe {
                element.link(self.head());
                self.tail().link(element);
            };
        }
    }

    /// Remove and return element from the beginning of the list
    /// Complexity: O(1)
    pub fn lpop(&mut self) -> Option<&Link> {
        if self.is_empty() {
            None
        } else {
            unsafe {
                let head = self.head_ptr();
                if self.tail_ptr() == head {
                    self.last = None;
                } else {
                    self.tail().link(self.head().next());
                }

                Some(&*head)
            }
        }
    }

    /// Returns the first element in the list without removing it
    /// Complexity: O(1)
    pub fn lpeek(&self) -> Option<&Link> {
        if self.is_empty() {
            None
        } else {
            Some(unsafe { self.head() })
        }
    }

    /// Inserts element at the end of the list
    /// Complexity: O(1)
    pub fn rpush(&mut self, element: &mut Link) {
        self.lpush(element);
        self.last = Some(unsafe { Link::new_linked(element) });
    }

    /// Remove and return element from the end of the list
    /// Complexity: O(1)
    pub fn rpop(&mut self) -> Option<&Link> {
        if self.is_empty() {
            None
        } else {
            let tail = unsafe { &*self.tail_ptr() };
            self.remove(tail)
        }
    }

    /// Returns the last element in the list without removing it
    /// Complexity: O(1)
    pub fn rpeek(&self) -> Option<&Link> {
        if self.is_empty() {
            None
        } else {
            Some(unsafe { self.tail() })
        }
    }

    /// Rotates list (first becomes last, second becomes first)
    /// Complexity: O(1)
    pub fn lpoprpush(&mut self) {
        if !self.is_empty() {
            unsafe { self.last().link(self.head()) };
        }
    }

    /// Find element
    /// Complexity: O(n)
    pub fn find(&self, element: &Link) -> Option<&Link> {
        unsafe { self.find_previous(element).and_then(|x| Some(x.next())) }
    }

    /// Remove and return element
    /// Complexity: O(n)
    pub fn remove(&mut self, element: &Link) -> Option<&Link> {
        if self.is_empty() {
            None
        } else if unsafe { self.head_ptr() } == element as *const _ {
            // this deals with the case of removing the only element,
            // at the cost of comparing head to element twice
            self.lpop()
        } else {
            unsafe {
                // storing element here so we can return it from the closure
                let res = element as *const _;
                if let Some(prev) = self.find_previous(element) {
                    prev.link(prev.next().next());
                    if self.tail_ptr() == res {
                        self.last().link(prev);
                    }
                    Some(&*res)
                } else {
                    None
                }
            }
        }
    }

    pub fn contains(&mut self, element: &Link) -> bool {
        unsafe { self.find_previous(element).is_some() }
    }

    pub fn iter(&self) -> Iter {
        let empty = self.is_empty();
        Iter {
            list: self,
            pos: if empty {
                core::ptr::null()
            } else {
                unsafe { self.head_ptr() }
            },
            stop: empty,
        }
    }

    pub fn iter_mut(&self) -> IterMut {
        let empty = self.is_empty();
        IterMut {
            list: self,
            pos: if empty {
                core::ptr::null()
            } else {
                unsafe { self.head_ptr() }
            },
            stop: empty,
        }
    }
}

impl core::fmt::Debug for List {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        if self.is_empty() {
            write!(f, "List {{}}")
        } else {
            unsafe {
                write!(
                    f,
                    "List {{ {:x} {:x}:{:x} {:x}:{:x}",
                    self.last().next_ptr() as usize,
                    self.tail() as *const _ as usize,
                    self.tail().next_ptr() as usize,
                    self.head() as *const _ as usize,
                    self.head().next_ptr() as usize,
                )
            }
        }
    }
}

/// internal
impl List {
    unsafe fn last(&self) -> &Link {
        &self.last.as_ref().unwrap_unchecked()
    }

    unsafe fn tail(&self) -> &Link {
        self.last().next()
    }

    unsafe fn tail_ptr(&self) -> *const Link {
        self.last().next_ptr()
    }

    unsafe fn head(&self) -> &Link {
        self.tail().next()
    }

    unsafe fn head_ptr(&self) -> *const Link {
        self.tail().next()
    }

    unsafe fn push_initial_element(&mut self, element: &mut Link) {
        element.link(element);
        self.last = Some(Link::new_linked(element));
    }

    unsafe fn find_previous(&self, element: &Link) -> Option<&Link> {
        if self.is_empty() {
            return None;
        }
        let mut pos = self.tail();
        let tail_ptr = pos as *const Link;
        let element_ptr = element as *const Link;
        loop {
            let next_ptr = pos.next_ptr();
            if next_ptr == element_ptr {
                return Some(pos);
            }
            if next_ptr == tail_ptr {
                return None;
            }
            pos = pos.next();
        }
    }
}

pub struct Iter<'a> {
    list: &'a List,
    pos: *const Link,
    stop: bool,
}

pub struct IterMut<'a> {
    list: &'a List,
    pos: *const Link,
    stop: bool,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Link;
    fn next(&mut self) -> Option<&'a Link> {
        if self.stop {
            None
        } else {
            unsafe {
                if self.list.tail_ptr() as *const _ == self.pos {
                    self.stop = true;
                }
                let res = &*self.pos;
                self.pos = res.next_ptr();
                Some(res)
            }
        }
    }
}

impl<'a> Iterator for IterMut<'a> {
    type Item = &'a Link;
    fn next(&mut self) -> Option<&'a Link> {
        if self.stop {
            None
        } else {
            unsafe {
                if self.list.tail_ptr() as *const _ == self.pos {
                    self.stop = true;
                }
                let res = &*self.pos;
                self.pos = res.next_ptr();
                Some(res)
            }
        }
    }
}

#[derive(Debug)]
pub struct TypedList<T, const OFFSET: usize> {
    list: List,
    _phantom: core::marker::PhantomData<T>,
}

impl<T, const OFFSET: usize> TypedList<T, { OFFSET }> {
    pub const fn new() -> Self {
        Self {
            list: List::new(),
            _phantom: core::marker::PhantomData {},
        }
    }

    pub fn is_empty(&mut self) -> bool {
        self.list.is_empty()
    }

    pub fn lpush(&mut self, element: &mut T) {
        let element = ((element as *mut T) as usize + OFFSET) as *mut Link;
        self.list.lpush(unsafe { &mut *element })
    }

    pub fn rpush(&mut self, element: &mut T) {
        let element = ((element as *mut T) as usize + OFFSET) as *mut Link;
        self.list.rpush(unsafe { &mut *element })
    }

    pub fn lpop(&mut self) -> Option<&mut T> {
        match self.list.lpop() {
            None => None,
            Some(link) => {
                Some(unsafe { &mut *((link as *const Link as usize - OFFSET) as *mut T) })
            }
        }
    }

    pub fn rpop(&mut self) -> Option<&mut T> {
        match self.list.rpop() {
            None => None,
            Some(link) => {
                Some(unsafe { &mut *((link as *const Link as usize - OFFSET) as *mut T) })
            }
        }
    }

    pub fn lpoprpush(&mut self) {
        self.list.lpoprpush()
    }

    pub fn remove(&mut self, element: &mut T) -> Option<&T> {
        let element = ((element as *mut T) as usize + OFFSET) as *mut Link;
        self.list
            .remove(unsafe { &mut *element })
            .and_then(|x| Some(unsafe { &*((x as *const Link as usize - OFFSET) as *mut T) }))
    }

    pub fn lpeek(&mut self) -> Option<&T> {
        match self.list.lpeek() {
            None => None,
            Some(link) => Some(unsafe { &*((link as *const Link as usize - OFFSET) as *mut T) }),
        }
    }

    pub fn rpeek(&mut self) -> Option<&T> {
        match self.list.rpeek() {
            None => None,
            Some(link) => Some(unsafe { &*((link as *const Link as usize - OFFSET) as *mut T) }),
        }
    }

    pub fn iter(&self) -> TypedIter<T> {
        TypedIter::<T> {
            iterator: self.list.iter(),
            offset: OFFSET,
            _phantom: core::marker::PhantomData::<T> {},
        }
    }

    pub fn iter_mut(&self) -> TypedIterMut<T> {
        TypedIterMut::<T> {
            iterator: self.list.iter(),
            offset: OFFSET,
            _phantom: core::marker::PhantomData::<T> {},
        }
    }
}

pub struct TypedIter<'a, T> {
    iterator: Iter<'a>,
    offset: usize,
    _phantom: core::marker::PhantomData<T>,
}

pub struct TypedIterMut<'a, T> {
    iterator: Iter<'a>,
    offset: usize,
    _phantom: core::marker::PhantomData<T>,
}

impl<'a, T: 'a> Iterator for TypedIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        if let Some(link) = self.iterator.next() {
            Some(unsafe { &*((link as *const Link as usize - self.offset) as *mut T) })
        } else {
            None
        }
    }
}

impl<'a, T: 'a> Iterator for TypedIterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<&'a mut T> {
        if let Some(link) = self.iterator.next() {
            Some(unsafe { &mut *((link as *const Link as usize - self.offset) as *mut T) })
        } else {
            None
        }
    }
}

// pub struct TypedIter<'a, T, const OFFSET: usize> {
//     iterator: Iter<'a>,
//     _phantom: core::marker::PhantomData<T>,
// }

// impl<'a, T: 'a, const OFFSET: usize> Iterator for TypedIter<'a, T, OFFSET> {
//     type Item = &'a T;

//     fn next(&mut self) -> Option<&'a T> {
//         if let Some(link) = self.iterator.next() {
//             Some(unsafe { &*((link as *const Link as usize - OFFSET) as *const T) })
//         } else {
//             None
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lpush_lpop_1() {
        let mut list = List::new();
        assert!(list.lpop().is_none());

        let mut node = Link::new();

        list.lpush(&mut node);

        assert!(unsafe { node.next_ptr() } == &node as *const Link);
        assert!(list.lpop().unwrap() as *const Link == &node as *const Link);
        assert!(list.lpop().is_none());
    }

    #[test]
    fn test_lpush_lpop_2() {
        let mut list = List::new();
        assert!(list.lpop().is_none());

        let mut node = Link::new();
        list.lpush(&mut node);
        assert!(unsafe { node.next_ptr() } == &node as *const Link);

        let mut node2 = Link::new();
        list.lpush(&mut node2);

        assert!(unsafe { node2.next_ptr() } == &node as *const Link);
        assert!(unsafe { node.next_ptr() } == &node2 as *const Link);
        assert!(unsafe { list.last().next_ptr() == &node as *const Link });

        assert!(list.lpop().unwrap() as *const Link == &node2 as *const Link);
        assert!(list.lpop().unwrap() as *const Link == &node as *const Link);
        assert!(list.lpop().is_none());
    }

    #[test]
    fn test_lpush_lpop_3() {
        let mut list = List::new();
        assert!(list.lpop().is_none());

        let mut node = Link::new();
        list.lpush(&mut node);
        assert!(unsafe { node.next_ptr() } == &node as *const Link);

        let mut node2 = Link::new();
        list.lpush(&mut node2);

        let mut node3 = Link::new();
        list.lpush(&mut node3);

        assert!(unsafe { node.next_ptr() } == &node3 as *const Link);
        assert!(unsafe { node2.next_ptr() } == &node as *const Link);
        assert!(unsafe { node3.next_ptr() } == &node2 as *const Link);
        assert!(unsafe { list.tail_ptr() == &node as *const Link });

        assert!(list.lpop().unwrap() as *const Link == &node3 as *const Link);
        assert!(unsafe { node.next_ptr() } == &node2 as *const Link);
        assert!(unsafe { node2.next_ptr() } == &node as *const Link);
        assert!(unsafe { list.tail_ptr() == &node as *const Link });
        //assert!(unsafe { node3.next_ptr() } == core::ptr::null());

        assert!(list.lpop().unwrap() as *const Link == &node2 as *const Link);
        assert!(unsafe { node.next_ptr() } == &node as *const Link);
        assert!(unsafe { list.tail_ptr() == &node as *const Link });
        //assert!(unsafe { node2.next_ptr() } == core::ptr::null());

        assert!(list.lpop().unwrap() as *const Link == &node as *const Link);
        //assert!(unsafe { node.next_ptr() } == core::ptr::null());
        assert!(list.last.is_none());

        assert!(list.lpop().is_none());
    }

    #[test]
    fn test_lpoprpush() {
        let mut list = List::new();

        let mut node = Link::new();
        let mut node2 = Link::new();

        list.lpush(&mut node);
        list.lpush(&mut node2);
        list.lpoprpush();

        assert!(list.lpop().unwrap() as *const _ == &node as *const _);
        assert!(list.lpop().unwrap() as *const _ == &node2 as *const _);
        assert!(list.lpop().is_none());
    }

    #[test]
    fn test_rpush() {
        let mut list = List::new();

        let mut node = Link::new();
        let mut node2 = Link::new();

        list.rpush(&mut node);
        list.rpush(&mut node2);

        assert!(list.lpop().unwrap() as *const _ == &mut node as *const _);
        assert!(list.lpop().unwrap() as *const _ == &mut node2 as *const _);
        assert!(list.lpop().is_none());
    }

    #[test]
    fn test_rpop() {
        let mut list = List::new();

        let mut node = Link::new();
        let mut node2 = Link::new();
        let mut node3 = Link::new();

        list.rpush(&mut node);
        list.rpush(&mut node2);
        list.rpush(&mut node3);

        assert!(unsafe { node.next_ptr() } == &node2 as *const Link);
        assert!(unsafe { node2.next_ptr() } == &node3 as *const Link);
        assert!(unsafe { node3.next_ptr() } == &node as *const Link);
        assert!(unsafe { list.tail_ptr() == &node3 as *const Link });

        assert!(list.rpop().unwrap() as *const _ == &mut node3 as *const _);
        assert!(unsafe { node.next_ptr() } == &node2 as *const Link);
        assert!(unsafe { node2.next_ptr() } == &node as *const Link);
        assert!(unsafe { list.tail_ptr() == &node2 as *const Link });

        assert!(list.rpop().unwrap() as *const _ == &mut node2 as *const _);
        assert!(unsafe { node.next_ptr() } == &node as *const Link);
        assert!(unsafe { list.tail_ptr() == &node as *const Link });

        assert!(list.rpop().unwrap() as *const _ == &mut node as *const _);
        assert!(list.is_empty());
        assert!(list.rpop().is_none());
    }

    #[test]
    fn test_remove_first() {
        let mut list = List::new();

        let mut node = Link::new();
        let mut node2 = Link::new();
        let mut node3 = Link::new();

        list.rpush(&mut node);
        list.rpush(&mut node2);
        list.rpush(&mut node3);

        assert!(list.remove(&node).is_some());

        assert!(list.rpop().unwrap() as *const _ == &mut node3 as *const _);
        assert!(list.rpop().unwrap() as *const _ == &mut node2 as *const _);
        assert!(list.rpop().is_none());
        assert!(list.lpop().is_none());
    }

    #[test]
    fn test_remove_mid() {
        let mut list = List::new();

        let mut node = Link::new();
        let mut node2 = Link::new();
        let mut node3 = Link::new();

        list.rpush(&mut node);
        list.rpush(&mut node2);
        list.rpush(&mut node3);

        assert!(list.remove(&node2).is_some());

        assert!(list.rpop().unwrap() as *const _ == &mut node3 as *const _);
        assert!(list.rpop().unwrap() as *const _ == &mut node as *const _);
        assert!(list.rpop().is_none());
        assert!(list.lpop().is_none());
    }

    #[test]
    fn test_remove_last() {
        let mut list = List::new();

        let mut node = Link::new();
        let mut node2 = Link::new();
        let mut node3 = Link::new();

        list.rpush(&mut node);
        list.rpush(&mut node2);
        list.rpush(&mut node3);

        assert!(list.remove(&node3).is_some());

        assert!(list.rpop().unwrap() as *const _ == &mut node2 as *const _);
        assert!(list.rpop().unwrap() as *const _ == &mut node as *const _);
        assert!(list.rpop().is_none());
        assert!(list.lpop().is_none());
    }

    #[test]
    fn test_iterator() {
        let mut list = List::new();

        let mut node = Link::new();
        let mut node2 = Link::new();
        let mut node3 = Link::new();

        list.rpush(&mut node);
        list.rpush(&mut node2);
        list.rpush(&mut node3);

        let pointers = [
            &mut node as *const Link,
            &mut node2 as *const Link,
            &mut node3 as *const Link,
        ];

        println!("pointers:");
        for entry in pointers.iter() {
            println!("{:x}", *entry as usize);
        }

        println!("list entries:");
        let mut i = 0;
        for entry in list.iter() {
            println!("{:x}", entry as *const Link as usize);
            assert_eq!(entry as *const Link, pointers[i]);
            i += 1;
        }
        assert_eq!(i, 3);
    }

    #[test]
    fn test_iterator_mut() {
        let mut list = List::new();

        let mut node = Link::new();
        let mut node2 = Link::new();
        let mut node3 = Link::new();

        list.rpush(&mut node);
        list.rpush(&mut node2);
        list.rpush(&mut node3);

        let pointers = [
            &mut node as *const Link,
            &mut node2 as *const Link,
            &mut node3 as *const Link,
        ];

        println!("pointers:");
        for entry in pointers.iter() {
            println!("{:x}", *entry as usize);
        }

        println!("list entries:");
        let mut i = 0;
        for entry in list.iter_mut() {
            println!("{:x}", entry as *const Link as usize);
            assert_eq!(entry as *const Link, pointers[i]);
            i += 1;
        }
        assert_eq!(i, 3);
    }

    #[test]
    fn test_iterator_empty() {
        let list = List::new();

        for _ in list.iter() {
            assert!(false);
        }
    }

    #[test]
    fn test_typed_iterator() {
        struct Data {
            data: u32,
            list_entry: Link,
        }

        let mut list: TypedList<Data, { offset_of!(Data, list_entry) }> = TypedList::new();

        let mut node = Data {
            data: 0,
            list_entry: Link::new(),
        };

        let mut node2 = Data {
            data: 1,
            list_entry: Link::new(),
        };

        let mut node3 = Data {
            data: 2,
            list_entry: Link::new(),
        };

        list.rpush(&mut node);
        list.rpush(&mut node2);
        list.rpush(&mut node3);

        let expected = [0 as u32, 1, 2];

        println!("list entries:");
        let mut i = 0;
        for entry in list.iter() {
            println!("{}", entry.data);
            assert_eq!(entry.data, expected[i]);
            i += 1;
        }
        assert_eq!(i, 3);
    }
}
