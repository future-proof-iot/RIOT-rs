//! Synchronization primitives.
mod channel;
mod lock;
mod mutex;

pub use channel::Channel;
pub use lock::Lock;
pub use mutex::{Mutex, MutexGuard};
