#![deny(clippy::pedantic)]

mod utils;

use proc_macro::TokenStream;

include!("config.rs");
include!("hw_setup.rs");
include!("read_sensor.rs");
include!("spawner.rs");
include!("task.rs");
include!("thread.rs");
