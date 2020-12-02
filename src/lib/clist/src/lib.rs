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
#![allow(incomplete_features)]
#![feature(const_generics)]
// features needed by our use of memoffset
#![feature(
    const_ptr_offset_from,
    const_raw_ptr_deref,
    raw_ref_macros,
    const_maybe_uninit_as_ptr
)]

extern crate memoffset;
pub use memoffset::offset_of;

#[derive(Clone, Copy, Debug)]
pub struct Link {
    next: *mut Link,
}

#[derive(Debug)]
pub struct List {
    last: *mut Link,
}

unsafe impl Sync for List {}
unsafe impl Send for List {}
unsafe impl Sync for Link {}
unsafe impl Send for Link {}

impl Link {
    pub const fn new() -> Link {
        Link {
            next: core::ptr::null_mut(),
        }
    }

    fn set_next(&mut self, next: *mut Link) {
        self.next = next;
    }

    fn clean(&mut self) {
        self.next = core::ptr::null_mut();
    }
}

impl List {
    fn get_last(&self) -> &mut Link {
        unsafe { &mut *self.last }
    }

    fn set_only(&mut self, element: &mut Link) {
        element.next = element as *mut Link;
        self.last = element as *mut Link;
    }

    pub fn is_empty(&self) -> bool {
        self.last.is_null()
    }

    /// creates a new, empty list
    pub const fn new() -> Self {
        List {
            last: core::ptr::null_mut(),
        }
    }

    /// Inserts element at the beginning of the list
    /// Complexity: O(1)
    pub fn lpush(&mut self, element: &mut Link) {
        if self.is_empty() {
            self.set_only(element);
        } else {
            element.set_next(self.get_last().next);
            self.get_last().next = element as *mut Link;
        }
    }

    pub fn rpush(&mut self, element: &mut Link) {
        self.lpush(element);
        self.last = element as *mut Link;
    }

    fn find_prev(&mut self, element: &Link) -> Option<&mut Link> {
        let element_ptr = element as *const Link;
        let mut pos = self.get_last();
        let last = pos as *const Link;
        loop {
            if pos.next as *const Link == element_ptr {
                return Some(pos);
            }
            if pos.next as *const Link == last {
                return None;
            }
            pos = unsafe { &mut *pos.next };
        }
    }

    pub fn remove(&mut self, element: &mut Link) -> bool {
        if self.is_empty() {
            false
        } else {
            let element_ptr = element as *mut Link;
            let pos = self.get_last();
            if pos as *mut Link == element_ptr {
                self.rpop();
                true
            } else {
                if let Some(prev) = self.find_prev(element) {
                    prev.set_next(element.next);
                    element.clean();
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn lpop(&mut self) -> Option<&mut Link> {
        if self.is_empty() {
            None
        } else {
            let first = self.get_last().next;
            if first == self.last {
                self.last = core::ptr::null_mut();
            } else {
                self.get_last().next = unsafe { (&*first).next };
            }

            let first = unsafe { &mut *first };
            first.clean();

            Some(first)
        }
    }

    pub fn lpoprpush(&mut self) {
        if !self.is_empty() {
            self.last = self.get_last().next;
        }
    }

    pub fn lpeek(&self) -> Option<&Link> {
        if self.is_empty() {
            None
        } else {
            Some(unsafe { &*self.get_last().next })
        }
    }

    pub fn rpeek(&self) -> Option<&Link> {
        if self.is_empty() {
            None
        } else {
            Some(unsafe { &*self.last })
        }
    }

    pub fn rpop(&mut self) -> Option<&mut Link> {
        if self.is_empty() {
            None
        } else {
            let last = self.last;
            while self.get_last().next != last {
                self.lpoprpush();
            }
            self.lpop()
        }
    }

    pub fn iter(&self) -> Iter {
        Iter {
            list: self,
            pos: unsafe { &mut *self.get_last().next },
            stop: self.is_empty(),
        }
    }

    pub fn iter_mut(&mut self) -> IterMut {
        IterMut {
            // order matters
            pos: unsafe { &mut *self.get_last().next },
            stop: self.is_empty(),
            list: self,
        }
    }
}

pub struct Iter<'a> {
    list: &'a List,
    pos: *mut Link,
    stop: bool,
}

pub struct IterMut<'a> {
    list: &'a mut List,
    pos: *mut Link,
    stop: bool,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Link;
    fn next(&mut self) -> Option<&'a Link> {
        if self.stop {
            None
        } else {
            if self.list.get_last() as *mut Link == self.pos {
                self.stop = true;
            }
            let res = unsafe { &*self.pos };
            self.pos = res.next;
            Some(res)
        }
    }
}

impl<'a> Iterator for IterMut<'a> {
    type Item = &'a mut Link;
    fn next(&mut self) -> Option<&'a mut Link> {
        if self.stop {
            None
        } else {
            if self.list.get_last() as *mut Link == self.pos {
                self.stop = true;
            }
            let res = unsafe { &mut *self.pos };
            self.pos = res.next;
            Some(res)
        }
    }
}

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
            Some(link) => Some(unsafe { &mut *((link as *mut Link as usize - OFFSET) as *mut T) }),
        }
    }

    pub fn rpop(&mut self) -> Option<&mut T> {
        match self.list.rpop() {
            None => None,
            Some(link) => Some(unsafe { &mut *((link as *mut Link as usize - OFFSET) as *mut T) }),
        }
    }

    pub fn lpoprpush(&mut self) {
        self.list.lpoprpush()
    }

    pub fn remove(&mut self, element: &mut T) -> bool {
        let element = ((element as *mut T) as usize + OFFSET) as *mut Link;
        self.list.remove(unsafe { &mut *element })
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

    pub fn iter_mut(&mut self) -> TypedIterMut<T> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lpush_lpop() {
        let mut list = List::new();

        let mut node = Link {
            next: core::ptr::null_mut(),
        };

        list.lpush(&mut node);
        assert!(list.lpop().unwrap() as *const _ == &mut node as *const _);
        assert!(list.lpop().is_none());
    }

    #[test]
    fn test_lpushrpop() {
        let mut list = List::new();

        let mut node = Link::new();
        let mut node2 = Link::new();

        list.lpush(&mut node);
        list.lpush(&mut node2);
        list.lpoprpush();

        assert!(list.lpop().unwrap() as *const _ == &mut node as *const _);
        assert!(list.lpop().unwrap() as *const _ == &mut node2 as *const _);
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

        assert!(list.rpop().unwrap() as *const _ == &mut node3 as *const _);
        assert!(list.rpop().unwrap() as *const _ == &mut node2 as *const _);
        assert!(list.rpop().unwrap() as *const _ == &mut node as *const _);
        assert!(list.rpop().is_none());
        assert!(list.lpop().is_none());
    }

    struct TestStruct {
        foo: usize,
        bar: isize,
        list_entry: Link,
    }

    #[test]
    fn test_adapter() {
        const OFFSET: usize = offset_of!(TestStruct, list_entry);
        let mut list: TypedList<TestStruct, OFFSET> = TypedList::new();

        let mut test1 = TestStruct {
            foo: 111,
            bar: 111111,
            list_entry: Link::new(),
        };
        let mut test2 = TestStruct {
            foo: 222,
            bar: 222222,
            list_entry: Link::new(),
        };
        list.lpush(&mut test1);
        list.lpush(&mut test2);

        assert_eq!(list.lpop().unwrap().foo, 222);
        assert_eq!(list.rpop().unwrap().bar, 111111);
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

        assert!(list.remove(&mut node) == true);

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

        assert!(list.remove(&mut node2) == true);

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

        assert!(list.remove(&mut node3) == true);

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
    fn test_typed_iterator() {
        struct Data {
            data: u32,
            list_entry: Link,
        };

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
