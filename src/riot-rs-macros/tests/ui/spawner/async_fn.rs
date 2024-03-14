#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

// As the macro will fail, this import will not get used
#[allow(unused_imports)]
use riot_rs::embassy::Spawner;

// FAIL: spawner functions cannot be async
#[riot_rs::spawner(autostart)]
async fn main(spawner: Spawner) {}
