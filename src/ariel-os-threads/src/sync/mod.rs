//! Synchronization primitives.
mod channel;
mod event;
mod lock;

pub use channel::Channel;
pub use event::Event;
pub use lock::Lock;
