//! Event Group module
//!
//! TODO: currently fixed to u32, make generic.
use crate::lock::Lock;
use clist::Link;
use core::cell::UnsafeCell;

pub(crate) type SubscriberList = clist::TypedList<Subscriber, 0>;

pub struct EventGroup(cortex_m::interrupt::Mutex<UnsafeCell<EventGroupInner>>);

pub struct EventGroupInner {
    waiters: SubscriberList,
}

#[derive(Clone)]
pub enum SubscribeMode {
    None,
    Any(u32),
    All(u32),
}

// repr(C) to prevent sorting of the array. That's necessary to have
// the list_entry member first, for the TypedList type.
// Otherwise, it breaks. See https://github.com/Gilnaa/memoffset/issues/37.
#[repr(C)]
pub struct Subscriber {
    list_entry: Link,
    state: u32,
    mode: SubscribeMode,
    lock: Lock,
}

impl EventGroup {
    pub const fn new() -> EventGroup {
        EventGroup {
            0: cortex_m::interrupt::Mutex::new(UnsafeCell::new(EventGroupInner {
                waiters: SubscriberList::new(),
            })),
        }
    }

    fn get_inner_mut(&self, cs: &cortex_m::interrupt::CriticalSection) -> &mut EventGroupInner {
        unsafe { &mut *self.0.borrow(cs).get() }
    }

    pub fn subscribe(&self, subscriber: &mut Subscriber) {
        cortex_m::interrupt::free(|cs| {
            let inner = &mut self.get_inner_mut(cs);
            let waiters = &mut inner.waiters;
            waiters.rpush(subscriber);
        });
    }

    pub fn unsubscribe(&self, subscriber: &mut Subscriber) {
        cortex_m::interrupt::free(|cs| {
            let inner = &mut self.get_inner_mut(cs);
            let waiters = &mut inner.waiters;
            waiters.remove(subscriber);
        });
    }

    pub fn set(&self, mask: u32) {
        cortex_m::interrupt::free(|cs| {
            let inner = &mut self.get_inner_mut(cs);
            let waiters = &mut inner.waiters;
            for waiter in waiters.iter_mut() {
                waiter.state |= mask;
                if match waiter.mode {
                    SubscribeMode::Any(bits) => (waiter.state & bits != 0),
                    SubscribeMode::All(bits) => (waiter.state & bits == bits),
                    SubscribeMode::None => false,
                } {
                    waiter.lock.release();
                }
            }
        });
    }
}

impl Subscriber {
    pub fn new(event_group: &EventGroup) -> Subscriber {
        let mut subscription = Subscriber::new_unsubscribed();
        event_group.subscribe(&mut subscription);
        subscription
    }

    pub const fn new_unsubscribed() -> Subscriber {
        Subscriber {
            list_entry: Link::new(),
            state: 0,
            mode: SubscribeMode::None,
            lock: Lock::new_locked(),
        }
    }

    pub fn wait(&mut self, events: SubscribeMode) -> u32 {
        loop {
            if let Some(bits) = cortex_m::interrupt::free(|_| match events {
                SubscribeMode::Any(bits) => {
                    if self.state & bits != 0 {
                        let result = self.state & bits;
                        self.state &= !bits;
                        Some(result)
                    } else {
                        None
                    }
                }
                SubscribeMode::All(bits) => {
                    if self.state & bits == bits {
                        self.state &= !bits;
                        Some(bits)
                    } else {
                        None
                    }
                }
                SubscribeMode::None => {
                    self.state = 0;
                    Some(0)
                }
            }) {
                return bits;
            }
            cortex_m::interrupt::free(|_| self.mode = events.clone());
            self.lock.acquire();
        }
    }

    pub fn clear(&mut self, mask: u32) -> u32 {
        cortex_m::interrupt::free(|_| {
            let cleared = self.state & mask;
            self.state &= !mask;
            cleared
        })
    }
}

#[cfg(test)]
static mut STACK: [u8; 1024] = [0; 1024];

#[cfg(test)]
fn func(arg: usize) {
    let event_group = arg as *const EventGroup;
    let event_group = unsafe { &*event_group };
    event_group.set(0b1);
    event_group.set(0b1010);
    event_group.set(0b100);
    event_group.set(0b11111);
}

#[test_case]
fn test_event_group() {
    use crate::thread::{CreateFlags, Thread};

    let event_group = EventGroup::new();
    let mut subscriber = Subscriber::new(&event_group);

    unsafe {
        Thread::create(
            &mut STACK,
            func,
            &event_group as *const EventGroup as usize,
            1,
            CreateFlags::empty(),
        );
    }

    assert_eq!(subscriber.state, 0);
    assert_eq!(subscriber.wait(SubscribeMode::Any(0b1)), 0b1);
    assert_eq!(subscriber.state, 0);
    assert_eq!(subscriber.wait(SubscribeMode::Any(0b10)), 0b10);
    assert_eq!(subscriber.state, 0b1000);
    assert_eq!(subscriber.wait(SubscribeMode::Any(0b100)), 0b100);
    assert_eq!(subscriber.state, 0b1000);
    assert_eq!(subscriber.wait(SubscribeMode::All(0b111)), 0b111);
    assert_eq!(subscriber.state, 0b11000);
}
