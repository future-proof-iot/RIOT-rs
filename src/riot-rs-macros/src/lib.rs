#![deny(clippy::pedantic)]

mod utils;

use proc_macro::TokenStream;

include!("config.rs");
include!("define_stm32_drivers.rs");
include!("spawner.rs");
include!("task.rs");
include!("thread.rs");
