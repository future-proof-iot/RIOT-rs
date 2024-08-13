#![deny(clippy::pedantic)]

mod utils;

use proc_macro::TokenStream;

include!("config.rs");
include!("call_with_stm32_peripheral_list.rs");
include!("spawner.rs");
include!("task.rs");
include!("thread.rs");
