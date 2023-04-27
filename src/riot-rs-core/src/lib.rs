#![no_std]

pub use embedded_threads as thread;
pub use thread::lock::{self, Lock};
pub mod c;
pub mod mutex;
