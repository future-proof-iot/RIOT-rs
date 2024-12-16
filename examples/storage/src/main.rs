#![no_main]
#![no_std]
#![feature(impl_trait_in_assoc_type)]
#![feature(used_with_arg)]

use ariel_os::debug::{
    exit,
    log::{defmt, info},
    ExitCode,
};

// Imports for using [`ariel_os::storage`]
use ariel_os::storage;
use serde::{Deserialize, Serialize};

// Some object. For storing it, derive the serde Serialize / Deserialize traits.
#[derive(Serialize, Deserialize, Debug, defmt::Format)]
struct MyConfig {
    val_uno: heapless::String<64>,
    val_dos: u64,
}

#[ariel_os::task(autostart)]
async fn main() {
    info!("Hello from storage test!");

    // Storing a primitive type (e.g., u32)
    let value: Option<u32> = storage::get("counter").await.unwrap();
    let value = if let Some(value) = value {
        info!("got counter value {}", value);
        value
    } else {
        info!("no counter value. first time running this test? try rebooting.");
        0
    };

    if value > 10 {
        info!("counter value > 10, aborting test to safe flash cycles.");
        exit(ExitCode::SUCCESS);
    }

    storage::insert("counter", value + 1).await.unwrap();

    {
        // By getting the Storage mutex directly, changing e.g., a counter,
        // can be done atomically w.r.t. concurrent access from the same firmware:
        let mut s = storage::lock().await;
        let value: Option<u32> = s.get("counter").await.unwrap();
        let value = value.unwrap_or_default();
        s.insert("counter", value + 1).await.unwrap();
    }

    // Storing a string value
    // For insertion, a literal can be used.
    storage::insert("string_key", "string_value").await.unwrap();

    // Getting a string value
    if let Some(string) = storage::get::<heapless::String<64>>("string_key")
        .await
        .unwrap()
    {
        info!("got heapless string value {}", string);
    }
    if let Some(string) = storage::get::<arrayvec::ArrayString<64>>("string_key")
        .await
        .unwrap()
    {
        // no `defmt::Format` for arrayvec, so just print length
        info!("got arrayvec string value with len {}", string.len());
    }

    // Storing an object
    let cfg = MyConfig {
        val_uno: heapless::String::<64>::try_from("some value").unwrap(),
        val_dos: 99,
    };
    storage::insert("my_config", cfg).await.unwrap();

    // Getting an object
    // Type used for `get()` needs to match what was used for `insert()`.
    let cfg: Option<MyConfig> = storage::get("my_config").await.unwrap();
    if let Some(cfg) = cfg {
        info!("got cfg {:?}", cfg);
    }

    // Getting a value as raw bytes probably does not return what you want due
    // to the way postcard works
    let cfg_array: Option<arrayvec::ArrayVec<u8, 256>> = storage::get("my_config").await.unwrap();
    if let Some(cfg) = cfg_array.as_ref() {
        info!("got cfg as arrayvec: {:x}", cfg.as_slice());
    }

    // Same for byte arrays
    let cfg_array: Option<[u8; 10]> = storage::get("my_config").await.unwrap();
    if let Some(cfg) = cfg_array.as_ref() {
        info!("got cfg as array: {:x}", cfg);
    }

    // raw bytes
    let bytes: [u8; 5] = [0, 1, 2, 3, 4];
    storage::insert("some_raw_bytes", bytes).await.unwrap();

    let bytes: Option<[u8; 5]> = storage::get("some_raw_bytes").await.unwrap();
    if let Some(bytes) = bytes.as_ref() {
        info!("got bytes as array: {:x}", bytes);
    }

    let bytes: Option<heapless::Vec<u8, 256>> = storage::get("some_raw_bytes").await.unwrap();
    if let Some(bytes) = bytes.as_ref() {
        info!("got bytes as heapless vec arr: {:x}", bytes);
    }
    info!("bye from storage test!");

    exit(ExitCode::SUCCESS);
}
