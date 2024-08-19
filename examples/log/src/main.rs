#![no_main]
#![no_std]
#![feature(impl_trait_in_assoc_type)]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use riot_rs::debug::log::*;

#[riot_rs::task(autostart)]
async fn main() {
    defmt::trace!("trace log level enabled");
    defmt::debug!("debug log level enabled");
    defmt::info!("info log level enabled");
    defmt::warn!("warn log level enabled");
    defmt::error!("error log level enabled (just testing)");
}
