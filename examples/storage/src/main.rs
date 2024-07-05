#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use riot_rs::debug::println;

//#[cfg(builder = "nrf5340dk")]
use riot_rs::arch::peripherals;

#[cfg(builder = "nrf52840dk")]
riot_rs::define_peripherals!(MyPeripherals { nvmc: NVMC });

#[cfg(builder = "particle-xenon")]
riot_rs::define_peripherals!(MyPeripherals { nvmc: NVMC });

use riot_rs::storage::Storage;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct MyConfig {
    val_dos: u64,
    val_uno: heapless::String<64>,
}

#[riot_rs::task(autostart, peripherals)]
async fn main(p: MyPeripherals) {
    println!("Hello from storage test!");

    let flash = embassy_nrf::nvmc::Nvmc::new(p.nvmc);
    let flash = embassy_embedded_hal::adapter::BlockingAsync::new(flash);
    let mut storage = Storage::new(flash).await;

    let value: Option<u32> = storage.get("counter").await.unwrap();
    let value = if let Some(value) = value {
        println!("got value {}", value);
        value
    } else {
        0
    };

    let cfg = MyConfig {
        val_uno: heapless::String::<64>::try_from("mybarrocks").unwrap(),
        val_dos: 99,
    };
    storage.put("counter", value + 1).await.unwrap();
    storage.put("my_config", cfg).await.unwrap();

    let cfg: Option<MyConfig> = storage.get("my_config").await.unwrap();
    if let Some(value) = cfg {
        println!("got value {:?}", value);
    }
    let cfg_bytes: Result<Option<heapless::String::<100>>, _> = storage.get("my_config").await;
    println!("got cfg_bytes {:?}", cfg_bytes);
}
