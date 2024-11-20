#![no_main]
#![feature(impl_trait_in_assoc_type)]
#![feature(used_with_arg)]

// As the macro will fail, this import will not get used
#[allow(unused_imports)]
use ariel_os::usb::UsbBuilderHook;

// FAIL: misspelled hook name
#[ariel_os::task(autostart, usb_builder_hooook)]
async fn main() {}
