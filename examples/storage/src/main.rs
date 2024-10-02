#![no_main]
#![no_std]
#![feature(impl_trait_in_assoc_type)]
#![feature(used_with_arg)]

use riot_rs::debug::log::{defmt, info};

use riot_rs::storage;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, defmt::Format)]
struct MyConfig {
    val_uno: heapless::String<64>,
    val_dos: u64,
}

#[riot_rs::task(autostart)]
async fn main() {
    info!("Hello from storage test!");
    //storage::del("counter").await.unwrap();
    let value: Option<u32> = storage::get("counter").await.unwrap();
    let value = if let Some(value) = value {
        info!("got value {}", value);
        value
    } else {
        0
    };

    let cfg = MyConfig {
        val_uno: heapless::String::<64>::try_from("mybarrocks").unwrap(),
        val_dos: 99,
    };
    storage::put("counter", value + 1).await.unwrap();
    storage::put("my_config", cfg).await.unwrap();

    {
        let mut s = storage::get_mutex().await;
        let value: Option<u32> = s.get("counter").await.unwrap();
        let value = value.unwrap_or_default();
        s.put("counter", value + 1).await.unwrap();
    }

    let cfg: Option<MyConfig> = storage::get("my_config").await.unwrap();
    if let Some(cfg) = cfg {
        info!("got cfg {:?}", cfg);
    }
}
