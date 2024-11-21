#![no_main]
#![no_std]
#![feature(impl_trait_in_assoc_type)]
#![feature(used_with_arg)]

use ariel_os::debug::log::*;

#[ariel_os::task(autostart)]
async fn main() {
    trace!("-- trace log level enabled");
    debug!("-- debug log level enabled");
    info!("-- info log level enabled");
    warn!("-- warn log level enabled");
    error!("-- error log level enabled (just testing)");
}
